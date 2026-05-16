use axum::{
    extract::{Extension, Path, State},
    response::Json,
    routing::{get, post},
    Router,
};
use chrono::{TimeZone, Utc};
use mongodb::bson::{doc, DateTime as BsonDt};
use serde_json::json;

use crate::app_state::AppState;
use crate::auth::middleware::AuthIdentity;
use crate::cache::template as tcache;
use crate::db::templates;
use crate::error::{ApiError, ApiResult};
use crate::models::template::TemplateDoc;
use crate::templates::render;
use crate::validate;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/templates", post(create).get(list))
        .route(
            "/templates/:name",
            get(get_one).put(update).delete(delete),
        )
}

fn view(t: &TemplateDoc, include_html: bool) -> ApiResult<serde_json::Value> {
    let created_at = Utc
        .timestamp_millis_opt(t.created_at.timestamp_millis())
        .single()
        .ok_or_else(|| ApiError::internal("bad created_at"))?;
    let updated_at = Utc
        .timestamp_millis_opt(t.updated_at.timestamp_millis())
        .single()
        .ok_or_else(|| ApiError::internal("bad updated_at"))?;
    let mut j = json!({
        "name":       t.name,
        "subject":    t.subject,
        "variables":  t.variables,
        "created_at": created_at,
        "updated_at": updated_at,
    });
    if include_html {
        j.as_object_mut()
            .unwrap()
            .insert("html".to_string(), json!(t.html));
    }
    Ok(j)
}

async fn create(
    State(state): State<AppState>,
    Extension(identity): Extension<AuthIdentity>,
    Json(req): Json<shared::CreateTemplateRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    if !validate::is_valid_resource_name(&req.name) {
        return Err(ApiError::bad_request(
            "invalid_name",
            "name must match ^[a-zA-Z0-9_-]{1,64}$",
        ));
    }
    if req.subject.chars().count() > state.limits.max_subject_chars {
        return Err(ApiError::bad_request(
            "subject_too_long",
            format!("subject must be ≤ {} chars", state.limits.max_subject_chars),
        ));
    }
    if req.html.len() > state.limits.max_html_bytes {
        return Err(ApiError::payload_too_large(format!(
            "html exceeds {} bytes",
            state.limits.max_html_bytes
        )));
    }
    render::validate_template(&req.subject)
        .map_err(|e| ApiError::bad_request("invalid_template", e.to_string()))?;
    render::validate_template(&req.html)
        .map_err(|e| ApiError::bad_request("invalid_template", e.to_string()))?;

    let count = templates::count(&state.db.db, &identity.user_id).await?;
    if count >= state.limits.max_templates {
        return Err(ApiError::bad_request(
            "limit_reached",
            format!("max {} templates per account", state.limits.max_templates),
        ));
    }

    let variables = render::extract_variables(&[&req.subject, &req.html]);
    let now = BsonDt::now();
    let d = TemplateDoc {
        id: None,
        user_id: identity.user_id,
        name: req.name,
        subject: req.subject,
        html: req.html,
        variables,
        created_at: now,
        updated_at: now,
    };
    match templates::insert(&state.db.db, &d).await {
        Ok(_) => {}
        Err(e) => {
            if e.to_string().contains("E11000") {
                return Err(ApiError::conflict(
                    "name_taken",
                    "template name already exists",
                ));
            }
            return Err(e.into());
        }
    }
    Ok(Json(view(&d, false)?))
}

async fn list(
    State(state): State<AppState>,
    Extension(identity): Extension<AuthIdentity>,
) -> ApiResult<Json<serde_json::Value>> {
    let items = templates::list(&state.db.db, &identity.user_id).await?;
    let mut out = Vec::with_capacity(items.len());
    for t in &items {
        out.push(view(t, false)?);
    }
    Ok(Json(json!(out)))
}

async fn get_one(
    State(state): State<AppState>,
    Extension(identity): Extension<AuthIdentity>,
    Path(name): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let t = templates::find(&state.db.db, &identity.user_id, &name)
        .await?
        .ok_or_else(|| ApiError::not_found("template not found"))?;
    Ok(Json(view(&t, true)?))
}

async fn update(
    State(state): State<AppState>,
    Extension(identity): Extension<AuthIdentity>,
    Path(name): Path<String>,
    Json(req): Json<shared::UpdateTemplateRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let existing = templates::find(&state.db.db, &identity.user_id, &name)
        .await?
        .ok_or_else(|| ApiError::not_found("template not found"))?;

    let new_subject = req.subject.unwrap_or(existing.subject);
    let new_html = req.html.unwrap_or(existing.html);

    if new_subject.chars().count() > state.limits.max_subject_chars {
        return Err(ApiError::bad_request(
            "subject_too_long",
            format!("subject must be ≤ {} chars", state.limits.max_subject_chars),
        ));
    }
    if new_html.len() > state.limits.max_html_bytes {
        return Err(ApiError::payload_too_large(format!(
            "html exceeds {} bytes",
            state.limits.max_html_bytes
        )));
    }
    render::validate_template(&new_subject)
        .map_err(|e| ApiError::bad_request("invalid_template", e.to_string()))?;
    render::validate_template(&new_html)
        .map_err(|e| ApiError::bad_request("invalid_template", e.to_string()))?;

    let variables = render::extract_variables(&[&new_subject, &new_html]);
    let set = doc! {
        "subject": &new_subject,
        "html": &new_html,
        "variables": &variables,
        "updated_at": BsonDt::now(),
    };
    let matched = templates::update(&state.db.db, &identity.user_id, &name, set).await?;
    if !matched {
        return Err(ApiError::not_found("template not found"));
    }
    let _ = tcache::invalidate(&state.redis, &identity.user_id.to_hex(), &name).await;

    let t = templates::find(&state.db.db, &identity.user_id, &name)
        .await?
        .ok_or_else(|| ApiError::internal("template vanished after update"))?;
    Ok(Json(view(&t, true)?))
}

async fn delete(
    State(state): State<AppState>,
    Extension(identity): Extension<AuthIdentity>,
    Path(name): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let deleted = templates::delete(&state.db.db, &identity.user_id, &name).await?;
    if !deleted {
        return Err(ApiError::not_found("template not found"));
    }
    let _ = tcache::invalidate(&state.redis, &identity.user_id.to_hex(), &name).await;
    Ok(Json(json!({ "status": "success" })))
}
