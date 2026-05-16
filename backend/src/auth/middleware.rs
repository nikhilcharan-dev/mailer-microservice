use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};
use mongodb::bson::oid::ObjectId;

use crate::app_state::AppState;
use crate::cache::api_key::{self as ak_cache, CachedApiKey};
use crate::db::users;
use crate::error::{ApiError, ApiResult};

/// Identity extracted by the auth middleware and attached to the request.
#[derive(Clone, Debug)]
pub struct AuthIdentity {
    pub user_id: ObjectId,
    pub username: String,
    pub daily_limit: u32,
}

fn parse_bearer(headers: &axum::http::HeaderMap) -> Option<String> {
    let h = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    Some(h.strip_prefix("Bearer ").unwrap_or(h).trim().to_string())
}

/// JWT-based auth for dashboard endpoints.
pub async fn jwt_guard(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> ApiResult<Response> {
    let token = parse_bearer(request.headers())
        .ok_or_else(|| ApiError::unauthorized("missing bearer token"))?;
    let claims = state
        .jwt
        .verify(&token)
        .map_err(|_| ApiError::unauthorized("invalid or expired token"))?;
    let user_id = ObjectId::parse_str(&claims.sub)
        .map_err(|_| ApiError::unauthorized("invalid subject in token"))?;

    let user = users::find_by_id(&state.db.db, &user_id)
        .await?
        .ok_or_else(|| ApiError::unauthorized("user no longer exists"))?;

    request.extensions_mut().insert(AuthIdentity {
        user_id,
        username: user.username,
        daily_limit: user.daily_limit,
    });
    Ok(next.run(request).await)
}

/// API-key auth for /v1/send/* — also checks the URL :username matches the key owner.
pub async fn api_key_guard(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> ApiResult<Response> {
    let key = parse_bearer(request.headers())
        .ok_or_else(|| ApiError::unauthorized("missing API key"))?;

    // Try cache first
    let identity: AuthIdentity = if let Some(cached) = ak_cache::get(&state.redis, &key).await? {
        let oid = ObjectId::parse_str(&cached.user_id)
            .map_err(|_| ApiError::unauthorized("invalid cached id"))?;
        AuthIdentity {
            user_id: oid,
            username: cached.username,
            daily_limit: cached.daily_limit,
        }
    } else {
        let user = users::find_by_api_key(&state.db.db, &key)
            .await?
            .ok_or_else(|| ApiError::unauthorized("invalid API key"))?;
        let oid = user.id.ok_or_else(|| ApiError::internal("user without id"))?;
        let identity = AuthIdentity {
            user_id: oid,
            username: user.username.clone(),
            daily_limit: user.daily_limit,
        };
        let _ = ak_cache::set(
            &state.redis,
            &key,
            &CachedApiKey {
                user_id: oid.to_hex(),
                username: user.username,
                daily_limit: user.daily_limit,
            },
        )
        .await;
        identity
    };

    request.extensions_mut().insert(identity);
    Ok(next.run(request).await)
}
