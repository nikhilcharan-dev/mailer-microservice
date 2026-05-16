mod app_state;
mod auth;
mod cache;
mod crypto;
mod db;
mod error;
mod mailer;
mod models;
mod routes;
mod templates;
mod validate;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    extract::State,
    http::{HeaderValue, Method},
    middleware,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use serde_json::json;
use tower_http::cors::CorsLayer;
use tower_http::limit::RequestBodyLimitLayer;

use app_state::{AppState, Limits};
use auth::{jwt::JwtConfig, middleware as auth_mw};
use crypto::aes::EncryptionKeyring;
use db::Db;

const MAX_BODY_BYTES: usize = 256 * 1024;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,central_mailer_backend=debug".into()),
        )
        .init();

    dotenvy::dotenv().ok();

    let db = Arc::new(Db::connect().await?);
    db.ensure_indexes().await?;
    tracing::info!("[mongodb] connected, indexes ensured");

    let redis = cache::connect().await?;
    tracing::info!("[redis] connected");

    let keyring = Arc::new(EncryptionKeyring::from_env()?);
    let jwt = Arc::new(JwtConfig::from_env()?);
    let limits = Arc::new(Limits::from_env());

    let state = AppState {
        db: db.clone(),
        redis: redis.clone(),
        keyring,
        jwt,
        limits,
    };

    // CORS — dashboard origin only
    let cors = match std::env::var("DASHBOARD_ORIGIN").ok() {
        Some(origin) => CorsLayer::new()
            .allow_origin(
                origin
                    .parse::<HeaderValue>()
                    .map_err(|e| anyhow::anyhow!("invalid DASHBOARD_ORIGIN: {e}"))?,
            )
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_headers([
                axum::http::header::AUTHORIZATION,
                axum::http::header::CONTENT_TYPE,
            ]),
        None => CorsLayer::new(),
    };

    // ── /v1/send (API-key auth, no CORS) ─────────────────────────────────────
    let send_routes = routes::send::router()
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_mw::api_key_guard,
        ));

    // ── /v1 dashboard routes (JWT auth, CORS-enabled) ────────────────────────
    let jwt_routes = Router::new()
        .merge(routes::account::router())
        .merge(routes::transports::router())
        .merge(routes::templates::router())
        .merge(routes::logs::router())
        .merge(routes::auth::protected_router())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_mw::jwt_guard,
        ));

    // ── /v1 public routes (signup, signin) ───────────────────────────────────
    let public_routes = routes::auth::public_router();

    let v1 = Router::new()
        .merge(public_routes)
        .merge(jwt_routes)
        .merge(send_routes)
        .layer(cors);

    let app = Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .nest("/v1", v1)
        .layer(RequestBodyLimitLayer::new(MAX_BODY_BYTES))
        .with_state(state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("[server] listening on {addr}");

    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({
        "status": "UP",
        "service": "central-mailer",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

async fn ready(State(state): State<AppState>) -> impl IntoResponse {
    let mongo_ok = state.db.ping().await.is_ok();
    let redis_ok = cache::ping(&state.redis).await.is_ok();
    let status = if mongo_ok && redis_ok {
        axum::http::StatusCode::OK
    } else {
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    };
    (
        status,
        Json(json!({
            "mongo": mongo_ok,
            "redis": redis_ok,
        })),
    )
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install ctrl-c handler");
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("[server] shutdown signal received, draining…");
}
