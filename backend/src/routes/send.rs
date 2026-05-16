use axum::{
    extract::{Extension, Path, State},
    response::Json,
    routing::post,
    Router,
};
use mongodb::bson::DateTime as BsonDt;
use serde_json::{json, Value};
use std::time::Instant;

use crate::app_state::AppState;
use crate::auth::middleware::AuthIdentity;
use crate::cache::{rate_limit, template as tplcache, transport as trcache};
use crate::db::{logs, templates, transports};
use crate::error::{ApiError, ApiResult};
use crate::mailer::smtp::{self, SendOutcome};
use crate::models::log::LogDoc;
use crate::templates::render;

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/send/:username/:transport/:template",
        post(send_handler),
    )
}

async fn send_handler(
    State(state): State<AppState>,
    Extension(identity): Extension<AuthIdentity>,
    Path((username, transport_name, template_name)): Path<(String, String, String)>,
    Json(body): Json<Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let started = Instant::now();

    // 1. Verify URL :username matches the API key owner
    if username != identity.username {
        return Err(ApiError::forbidden(
            "API key does not own this username",
        ));
    }

    // 2. Atomic rate-limit reservation
    let uid_hex = identity.user_id.to_hex();
    let n = rate_limit::reserve_daily(&state.redis, &uid_hex).await?;
    if n as u32 > identity.daily_limit {
        rate_limit::release_daily(&state.redis, &uid_hex).await;
        return Err(ApiError::rate_limited(
            "rate_limited",
            "daily send limit reached",
        ));
    }

    // 3. Resolve template (cache → negcache → MongoDB)
    let template = match load_template(&state, &uid_hex, &template_name).await {
        Ok(Some(t)) => t,
        Ok(None) => {
            rate_limit::release_daily(&state.redis, &uid_hex).await;
            return Err(ApiError::not_found("template not found"));
        }
        Err(e) => {
            rate_limit::release_daily(&state.redis, &uid_hex).await;
            return Err(e);
        }
    };

    // 4. Validate that all required template variables exist in body
    let obj = body
        .as_object()
        .ok_or_else(|| ApiError::bad_request("invalid_body", "body must be a JSON object"))?;
    let to = obj
        .get("to")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::bad_request("missing_to", "'to' field is required"))?
        .to_string();
    if !crate::validate::is_email(&to) {
        rate_limit::release_daily(&state.redis, &uid_hex).await;
        return Err(ApiError::bad_request("invalid_to", "'to' is not a valid email"));
    }
    let reserved = ["to", "subject"];
    for var in &template.variables {
        if reserved.contains(&var.as_str()) {
            continue;
        }
        match obj.get(var) {
            Some(v) if !v.is_null() && (!v.is_string() || !v.as_str().unwrap().is_empty()) => {}
            _ => {
                rate_limit::release_daily(&state.redis, &uid_hex).await;
                return Err(ApiError::bad_request(
                    "missing_variable",
                    format!("missing template variable: {var}"),
                ));
            }
        }
    }

    // 5. Resolve transport (cache → MongoDB → decrypt → cache)
    let transport_cfg = match load_transport(&state, &uid_hex, &transport_name).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            rate_limit::release_daily(&state.redis, &uid_hex).await;
            return Err(ApiError::not_found("transport not found"));
        }
        Err(e) => {
            rate_limit::release_daily(&state.redis, &uid_hex).await;
            return Err(e);
        }
    };

    // 6. Render template
    let subject_override = obj.get("subject").and_then(|v| v.as_str());
    let subject_template = subject_override.unwrap_or(&template.subject);
    let rendered_subject = render::render(subject_template, &body)
        .map_err(|e| ApiError::bad_request("render_failed", e.to_string()))?;
    let rendered_html = render::render(&template.html, &body)
        .map_err(|e| ApiError::bad_request("render_failed", e.to_string()))?;

    // 7. Build transport + send with timeout
    let smtp_transport = smtp::build_transport(&transport_cfg)
        .map_err(|e| ApiError::internal(format!("transport build failed: {e}")))?;
    let outcome = smtp::send(
        &smtp_transport,
        &transport_cfg,
        &to,
        &rendered_subject,
        rendered_html,
    )
    .await;

    let duration_ms = started.elapsed().as_millis().min(u32::MAX as u128) as u32;
    let (status_str, error_str, response): (&str, Option<String>, ApiResult<Json<Value>>) = match &outcome {
        SendOutcome::Ok(mid) => (
            "success",
            None,
            Ok(Json(json!({ "status": "success", "message_id": mid }))),
        ),
        SendOutcome::Failed(e) => (
            "failed",
            Some(e.clone()),
            Err(ApiError::new(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "smtp_failed",
                "SMTP delivery failed",
            )),
        ),
        SendOutcome::TimedOut => (
            "failed",
            Some("timeout".to_string()),
            Err(ApiError::new(
                axum::http::StatusCode::GATEWAY_TIMEOUT,
                "smtp_timeout",
                "SMTP send timed out",
            )),
        ),
    };

    // 8. Async side-effects: release on failure, write log
    let was_failed = !matches!(outcome, SendOutcome::Ok(_));
    let redis = state.redis.clone();
    let db = state.db.clone();
    let user_id = identity.user_id;
    let uid_hex_clone = uid_hex.clone();
    let transport_name_clone = transport_name.clone();
    let template_name_clone = template_name.clone();
    let to_clone = to.clone();
    let error_clone = error_str.clone();
    let status_owned = status_str.to_string();
    tokio::spawn(async move {
        if was_failed {
            rate_limit::release_daily(&redis, &uid_hex_clone).await;
        }
        let log_doc = LogDoc {
            id: None,
            user_id,
            transport_name: transport_name_clone,
            template_name: template_name_clone,
            to: to_clone,
            status: status_owned,
            error: error_clone,
            duration_ms,
            sent_at: BsonDt::now(),
        };
        if let Err(e) = logs::insert(&db.db, &log_doc).await {
            tracing::warn!("delivery log write failed: {e}");
        }
    });

    response
}

async fn load_template(
    state: &AppState,
    uid_hex: &str,
    name: &str,
) -> ApiResult<Option<tplcache::CachedTemplate>> {
    if let Some(t) = tplcache::get(&state.redis, uid_hex, name).await? {
        return Ok(Some(t));
    }
    if tplcache::negcache_hit(&state.redis, uid_hex, name).await? {
        return Ok(None);
    }
    let oid = mongodb::bson::oid::ObjectId::parse_str(uid_hex)
        .map_err(|_| ApiError::internal("bad user id"))?;
    let t = templates::find(&state.db.db, &oid, name).await?;
    match t {
        Some(doc) => {
            let c = tplcache::CachedTemplate {
                subject: doc.subject,
                html: doc.html,
                variables: doc.variables,
            };
            let _ = tplcache::set(&state.redis, uid_hex, name, &c).await;
            Ok(Some(c))
        }
        None => {
            let _ = tplcache::negcache_set(&state.redis, uid_hex, name).await;
            Ok(None)
        }
    }
}

async fn load_transport(
    state: &AppState,
    uid_hex: &str,
    name: &str,
) -> ApiResult<Option<trcache::CachedTransport>> {
    if let Some(t) = trcache::get(&state.redis, uid_hex, name).await? {
        return Ok(Some(t));
    }
    if trcache::negcache_hit(&state.redis, uid_hex, name).await? {
        return Ok(None);
    }
    let oid = mongodb::bson::oid::ObjectId::parse_str(uid_hex)
        .map_err(|_| ApiError::internal("bad user id"))?;
    let t = transports::find(&state.db.db, &oid, name).await?;
    match t {
        Some(doc) => {
            let pass = state
                .keyring
                .decrypt(
                    &doc.smtp_pass_enc.bytes,
                    &doc.smtp_pass_nonce.bytes,
                    doc.key_version,
                )
                .map_err(|e| {
                    tracing::error!("decrypt: {e}");
                    ApiError::internal("could not decrypt credentials")
                })?;
            let pass = String::from_utf8(pass)
                .map_err(|_| ApiError::internal("decrypted pass not utf-8"))?;
            let c = trcache::CachedTransport {
                smtp_host: doc.smtp_host,
                smtp_port: doc.smtp_port,
                tls_mode: doc.tls_mode,
                smtp_user: doc.smtp_user,
                smtp_pass: pass,
                from_name: doc.from_name,
                from_email: doc.from_email,
            };
            let _ = trcache::set(&state.redis, uid_hex, name, &c).await;
            Ok(Some(c))
        }
        None => {
            let _ = trcache::negcache_set(&state.redis, uid_hex, name).await;
            Ok(None)
        }
    }
}
