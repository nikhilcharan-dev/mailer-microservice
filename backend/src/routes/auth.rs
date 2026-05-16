use axum::{
    extract::{ConnectInfo, Extension, State},
    response::Json,
    routing::post,
    Router,
};
use mongodb::bson::DateTime as BsonDt;
use serde_json::json;
use std::net::SocketAddr;

use crate::app_state::AppState;
use crate::auth::{api_key, middleware::AuthIdentity};
use crate::cache::{api_key as ak_cache, rate_limit};
use crate::crypto::password;
use crate::db::users;
use crate::error::{ApiError, ApiResult};
use crate::models::user::UserDoc;
use crate::validate;

pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/auth/signup", post(signup))
        .route("/auth/signin", post(signin))
}

pub fn protected_router() -> Router<AppState> {
    Router::new().route("/auth/apikey/rotate", post(rotate_api_key))
}

fn ip_str(addr: &SocketAddr) -> String {
    addr.ip().to_string()
}

async fn signup(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<shared::SignupRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let ip = ip_str(&addr);

    if !rate_limit::check_signup(&state.redis, &ip).await? {
        return Err(ApiError::rate_limited(
            "rate_limited",
            "too many signup attempts; try again later",
        ));
    }

    if !validate::is_valid_username(&req.username) {
        return Err(ApiError::bad_request(
            "invalid_username",
            "username must be 3-32 chars, lowercase alphanumeric + - or _, and not a reserved name",
        ));
    }
    if !validate::is_email(&req.email) {
        return Err(ApiError::bad_request("invalid_email", "invalid email format"));
    }
    if !validate::is_strong_password(&req.password) {
        return Err(ApiError::bad_request(
            "weak_password",
            "password must be at least 12 characters",
        ));
    }

    // Uniqueness pre-check (DB index also enforces)
    if users::find_by_username(&state.db.db, &req.username)
        .await?
        .is_some()
    {
        return Err(ApiError::conflict("username_taken", "username already taken"));
    }
    if users::find_by_email(&state.db.db, &req.email).await?.is_some() {
        return Err(ApiError::conflict("email_taken", "email already registered"));
    }

    let pass_hash = password::hash(&req.password).map_err(|e| {
        tracing::error!("password hash failed: {e}");
        ApiError::internal("could not hash password")
    })?;

    let api_key_val = api_key::generate();

    let user = UserDoc {
        id: None,
        username: req.username,
        email: req.email,
        password_hash: pass_hash,
        api_key: api_key_val.clone(),
        daily_limit: state.limits.default_daily,
        created_at: BsonDt::now(),
    };

    users::insert(&state.db.db, &user).await?;

    Ok(Json(json!({
        "status": "success",
        "api_key": api_key_val,
    })))
}

async fn signin(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<shared::SigninRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let ip = ip_str(&addr);
    if !rate_limit::check_signin(&state.redis, &ip).await? {
        return Err(ApiError::rate_limited(
            "rate_limited",
            "too many failed sign-in attempts; try again later",
        ));
    }

    let user = users::find_by_identifier(&state.db.db, &req.identifier).await?;
    let user = match user {
        Some(u) => u,
        None => {
            rate_limit::record_signin_failure(&state.redis, &ip).await?;
            return Err(ApiError::unauthorized("invalid credentials"));
        }
    };

    if !password::verify(&req.password, &user.password_hash) {
        rate_limit::record_signin_failure(&state.redis, &ip).await?;
        return Err(ApiError::unauthorized("invalid credentials"));
    }

    let user_id = user
        .id
        .ok_or_else(|| ApiError::internal("user without id"))?;
    let token = state
        .jwt
        .issue(&user_id.to_hex(), &user.username)
        .map_err(|e| {
            tracing::error!("jwt issue failed: {e}");
            ApiError::internal("could not issue token")
        })?;

    Ok(Json(json!({
        "status": "success",
        "token": token,
    })))
}

async fn rotate_api_key(
    State(state): State<AppState>,
    Extension(identity): Extension<AuthIdentity>,
) -> ApiResult<Json<serde_json::Value>> {
    let user = users::find_by_id(&state.db.db, &identity.user_id)
        .await?
        .ok_or_else(|| ApiError::internal("user disappeared"))?;
    let old_key = user.api_key;
    let new_key = api_key::generate();
    users::rotate_api_key(&state.db.db, &identity.user_id, &new_key).await?;
    let _ = ak_cache::invalidate(&state.redis, &old_key).await;
    Ok(Json(json!({
        "status": "success",
        "api_key": new_key,
    })))
}
