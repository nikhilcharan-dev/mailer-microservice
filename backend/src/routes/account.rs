use axum::{
    extract::{Extension, State},
    response::Json,
    routing::get,
    Router,
};
use chrono::{TimeZone, Utc};
use serde_json::json;

use crate::app_state::AppState;
use crate::auth::middleware::AuthIdentity;
use crate::cache::{api_key as ak_cache, rate_limit};
use crate::crypto::password;
use crate::db::{logs, templates, transports, users};
use crate::error::{ApiError, ApiResult};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/account", get(get_account).delete(delete_account))
}

async fn get_account(
    State(state): State<AppState>,
    Extension(identity): Extension<AuthIdentity>,
) -> ApiResult<Json<serde_json::Value>> {
    let user = users::find_by_id(&state.db.db, &identity.user_id)
        .await?
        .ok_or_else(|| ApiError::internal("user disappeared"))?;

    let sent_today = rate_limit::get_daily(&state.redis, &identity.user_id.to_hex()).await? as u64;
    let transport_count = transports::count(&state.db.db, &identity.user_id).await?;
    let template_count = templates::count(&state.db.db, &identity.user_id).await?;

    let created_at = Utc.timestamp_millis_opt(user.created_at.timestamp_millis()).single()
        .ok_or_else(|| ApiError::internal("bad created_at"))?;

    Ok(Json(json!({
        "username":        user.username,
        "email":           user.email,
        "daily_limit":     user.daily_limit,
        "sent_today":      sent_today,
        "transport_count": transport_count,
        "template_count":  template_count,
        "created_at":      created_at,
    })))
}

async fn delete_account(
    State(state): State<AppState>,
    Extension(identity): Extension<AuthIdentity>,
    Json(req): Json<shared::DeleteAccountRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let user = users::find_by_id(&state.db.db, &identity.user_id)
        .await?
        .ok_or_else(|| ApiError::internal("user disappeared"))?;
    if !password::verify(&req.password, &user.password_hash) {
        return Err(ApiError::unauthorized("password does not match"));
    }

    // Invalidate cached transports/templates names — best-effort
    let trs = transports::list(&state.db.db, &identity.user_id).await?;
    for t in &trs {
        let _ = crate::cache::transport::invalidate(
            &state.redis,
            &identity.user_id.to_hex(),
            &t.name,
        )
        .await;
    }
    let tpls = templates::list(&state.db.db, &identity.user_id).await?;
    for t in &tpls {
        let _ = crate::cache::template::invalidate(
            &state.redis,
            &identity.user_id.to_hex(),
            &t.name,
        )
        .await;
    }

    transports::delete_all_for_user(&state.db.db, &identity.user_id).await?;
    templates::delete_all_for_user(&state.db.db, &identity.user_id).await?;
    users::delete(&state.db.db, &identity.user_id).await?;
    let _ = ak_cache::invalidate(&state.redis, &user.api_key).await;

    // delivery_logs are retained for the TTL window (90d) for abuse investigation
    let _ = logs::count_since(&state.db.db, &identity.user_id, mongodb::bson::DateTime::now()).await;

    Ok(Json(json!({ "status": "success" })))
}
