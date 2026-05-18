use axum::{
    extract::{Extension, Path, State},
    response::Json,
    routing::{get, post},
    Router,
};
use chrono::{TimeZone, Utc};
use mongodb::bson::{doc, Binary, DateTime as BsonDt};
use serde::Deserialize;
use serde_json::json;

use crate::app_state::AppState;
use crate::auth::middleware::AuthIdentity;
use crate::cache::{rate_limit, transport as tcache};
use crate::db::transports;
use crate::error::{ApiError, ApiResult};
use crate::mailer::smtp;
use crate::models::transport::TransportDoc;
use crate::validate;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/transports", post(create).get(list))
        .route(
            "/transports/:name",
            get(get_one).put(update).delete(delete),
        )
        .route("/transports/:name/verify", post(verify_transport))
}

fn bson_binary(bytes: Vec<u8>) -> Binary {
    Binary {
        subtype: mongodb::bson::spec::BinarySubtype::Generic,
        bytes,
    }
}

fn to_view(t: &TransportDoc) -> ApiResult<serde_json::Value> {
    let created_at = Utc
        .timestamp_millis_opt(t.created_at.timestamp_millis())
        .single()
        .ok_or_else(|| ApiError::internal("bad created_at"))?;
    let updated_at = Utc
        .timestamp_millis_opt(t.updated_at.timestamp_millis())
        .single()
        .ok_or_else(|| ApiError::internal("bad updated_at"))?;
    Ok(json!({
        "name":       t.name,
        "smtp_host":  t.smtp_host,
        "smtp_port":  t.smtp_port,
        "tls_mode":   t.tls_mode,
        "smtp_user":  t.smtp_user,
        "from_name":  t.from_name,
        "from_email": t.from_email,
        "created_at": created_at,
        "updated_at": updated_at,
    }))
}

async fn create(
    State(state): State<AppState>,
    Extension(identity): Extension<AuthIdentity>,
    Json(req): Json<shared::CreateTransportRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    if !validate::is_valid_resource_name(&req.name) {
        return Err(ApiError::bad_request(
            "invalid_name",
            "name must match ^[a-zA-Z0-9_-]{1,64}$",
        ));
    }
    if !validate::is_valid_tls_mode(&req.tls_mode) {
        return Err(ApiError::bad_request(
            "invalid_tls_mode",
            "tls_mode must be one of: starttls, tls, none",
        ));
    }
    if !validate::is_email(&req.from_email) {
        return Err(ApiError::bad_request(
            "invalid_email",
            "from_email is not a valid address",
        ));
    }

    let count = transports::count(&state.db.db, &identity.user_id).await?;
    if count >= state.limits.max_transports {
        return Err(ApiError::bad_request(
            "limit_reached",
            format!(
                "max {} transports per account",
                state.limits.max_transports
            ),
        ));
    }

    let (ciphertext, nonce, version) = state
        .keyring
        .encrypt(req.smtp_pass.as_bytes())
        .map_err(|e| {
            tracing::error!("encrypt failed: {e}");
            ApiError::internal("encryption failed")
        })?;

    let now = BsonDt::now();
    let doc = TransportDoc {
        id: None,
        user_id: identity.user_id,
        name: req.name.clone(),
        smtp_host: req.smtp_host,
        smtp_port: req.smtp_port,
        tls_mode: req.tls_mode,
        smtp_user: req.smtp_user,
        smtp_pass_enc: bson_binary(ciphertext),
        smtp_pass_nonce: bson_binary(nonce),
        key_version: version,
        from_name: req.from_name,
        from_email: req.from_email,
        created_at: now,
        updated_at: now,
    };

    match transports::insert(&state.db.db, &doc).await {
        Ok(_) => {}
        Err(e) => {
            let s = e.to_string();
            if s.contains("E11000") {
                return Err(ApiError::conflict(
                    "name_taken",
                    "transport name already exists for this account",
                ));
            }
            tracing::error!("insert transport: {s}");
            return Err(ApiError::internal("database error"));
        }
    }

    Ok(Json(to_view(&doc)?))
}

async fn list(
    State(state): State<AppState>,
    Extension(identity): Extension<AuthIdentity>,
) -> ApiResult<Json<serde_json::Value>> {
    let items = transports::list(&state.db.db, &identity.user_id).await?;
    let mut out = Vec::with_capacity(items.len());
    for t in &items {
        out.push(to_view(t)?);
    }
    Ok(Json(json!(out)))
}

async fn get_one(
    State(state): State<AppState>,
    Extension(identity): Extension<AuthIdentity>,
    Path(name): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let t = transports::find(&state.db.db, &identity.user_id, &name)
        .await?
        .ok_or_else(|| ApiError::not_found("transport not found"))?;
    Ok(Json(to_view(&t)?))
}

async fn update(
    State(state): State<AppState>,
    Extension(identity): Extension<AuthIdentity>,
    Path(name): Path<String>,
    Json(req): Json<shared::UpdateTransportRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let mut set = doc! {};
    if let Some(v) = &req.smtp_host {
        set.insert("smtp_host", v);
    }
    if let Some(v) = req.smtp_port {
        set.insert("smtp_port", v as i32);
    }
    if let Some(v) = &req.tls_mode {
        if !validate::is_valid_tls_mode(v) {
            return Err(ApiError::bad_request("invalid_tls_mode", "bad tls_mode"));
        }
        set.insert("tls_mode", v);
    }
    if let Some(v) = &req.smtp_user {
        set.insert("smtp_user", v);
    }
    if let Some(v) = &req.smtp_pass {
        let (ct, nonce, version) = state.keyring.encrypt(v.as_bytes()).map_err(|e| {
            tracing::error!("encrypt: {e}");
            ApiError::internal("encryption failed")
        })?;
        set.insert("smtp_pass_enc", bson_binary(ct));
        set.insert("smtp_pass_nonce", bson_binary(nonce));
        set.insert("key_version", version);
    }
    if let Some(v) = &req.from_name {
        set.insert("from_name", v);
    }
    if let Some(v) = &req.from_email {
        if !validate::is_email(v) {
            return Err(ApiError::bad_request("invalid_email", "bad from_email"));
        }
        set.insert("from_email", v);
    }
    set.insert("updated_at", BsonDt::now());

    let matched = transports::update(&state.db.db, &identity.user_id, &name, set).await?;
    if !matched {
        return Err(ApiError::not_found("transport not found"));
    }
    let _ = tcache::invalidate(&state.redis, &identity.user_id.to_hex(), &name).await;

    let t = transports::find(&state.db.db, &identity.user_id, &name)
        .await?
        .ok_or_else(|| ApiError::internal("transport vanished after update"))?;
    Ok(Json(to_view(&t)?))
}

async fn delete(
    State(state): State<AppState>,
    Extension(identity): Extension<AuthIdentity>,
    Path(name): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let deleted = transports::delete(&state.db.db, &identity.user_id, &name).await?;
    if !deleted {
        return Err(ApiError::not_found("transport not found"));
    }
    let _ = tcache::invalidate(&state.redis, &identity.user_id.to_hex(), &name).await;
    Ok(Json(json!({ "status": "success" })))
}

#[derive(Deserialize, Default)]
struct VerifyRequest {
    test_to: Option<String>,
}

async fn verify_transport(
    State(state): State<AppState>,
    Extension(identity): Extension<AuthIdentity>,
    Path(name): Path<String>,
    body: Option<Json<VerifyRequest>>,
) -> ApiResult<Json<serde_json::Value>> {
    if !rate_limit::check_verify(&state.redis, &identity.user_id.to_hex()).await? {
        return Err(ApiError::rate_limited(
            "rate_limited",
            "too many verify attempts; wait a moment",
        ));
    }
    let t = transports::find(&state.db.db, &identity.user_id, &name)
        .await?
        .ok_or_else(|| ApiError::not_found("transport not found"))?;

    let pass = state
        .keyring
        .decrypt(&t.smtp_pass_enc.bytes, &t.smtp_pass_nonce.bytes, t.key_version)
        .map_err(|e| {
            tracing::error!("decrypt: {e}");
            ApiError::internal("could not decrypt credentials")
        })?;
    let pass = String::from_utf8(pass)
        .map_err(|_| ApiError::internal("decrypted password not utf-8"))?;

    let cfg = tcache::CachedTransport {
        smtp_host: t.smtp_host,
        smtp_port: t.smtp_port,
        tls_mode: t.tls_mode,
        smtp_user: t.smtp_user,
        smtp_pass: pass,
        from_name: t.from_name.clone(),
        from_email: t.from_email.clone(),
    };

    let test_to = body.and_then(|b| b.0.test_to);

    if let Some(ref to) = test_to {
        // Send a real test email through the transport
        let transport = smtp::build_transport(&cfg).map_err(|e| {
            ApiError::internal(format!("build transport: {e}"))
        })?;
        let html = format!(
            "<div style=\"font-family:sans-serif;max-width:480px;margin:40px auto;\">\
             <h2 style=\"color:#6366f1;\">☁ cloudMailer — Test Email</h2>\
             <p>This test message was sent through transport <strong>{name}</strong>.</p>\
             <p>If you received this, your SMTP configuration is working correctly.</p>\
             <hr style=\"border:none;border-top:1px solid #e5e7eb;margin:24px 0;\"/>\
             <p style=\"color:#9ca3af;font-size:12px;\">Sent by cloudMailer verify endpoint</p>\
             </div>"
        );
        match smtp::send(&transport, &cfg, to, "☁ cloudMailer — Test Email", html).await {
            smtp::SendOutcome::Ok(_) => Ok(Json(json!({
                "status": "success",
                "ok": true,
                "sent_to": to
            }))),
            smtp::SendOutcome::Failed(e) => Ok(Json(json!({
                "status": "error",
                "ok": false,
                "error": e
            }))),
            smtp::SendOutcome::TimedOut => Ok(Json(json!({
                "status": "error",
                "ok": false,
                "error": "SMTP send timed out"
            }))),
        }
    } else {
        // Connection-only test (no email sent)
        match smtp::verify(&cfg).await {
            Ok(_) => Ok(Json(json!({ "status": "success", "ok": true }))),
            Err(e) => Ok(Json(
                json!({ "status": "error", "ok": false, "error": e.to_string() }),
            )),
        }
    }
}
