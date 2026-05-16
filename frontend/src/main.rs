mod api;
mod pages;
mod session;
mod ui;

use std::net::SocketAddr;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,central_mailer_frontend=debug".into()),
        )
        .init();

    dotenvy::dotenv().ok();

    let app = Router::new()
        // ── Auth (public) ────────────────────────────────────────────────────
        .route(
            "/login",
            get(pages::auth::login_page).post(pages::auth::login_submit),
        )
        .route(
            "/signup",
            get(pages::auth::signup_page).post(pages::auth::signup_submit),
        )
        .route("/logout", post(pages::auth::logout))
        // ── Dashboard ────────────────────────────────────────────────────────
        .route("/", get(pages::dashboard::dashboard))
        // ── Transports ───────────────────────────────────────────────────────
        .route(
            "/transports",
            get(pages::transports::list_page).post(pages::transports::create),
        )
        .route(
            "/transports/:name/delete",
            post(pages::transports::delete),
        )
        .route(
            "/transports/:name/verify",
            post(pages::transports::verify),
        )
        // ── Templates ────────────────────────────────────────────────────────
        .route(
            "/templates",
            get(pages::templates::list_page).post(pages::templates::create),
        )
        .route(
            "/templates/:name",
            get(pages::templates::detail_page).post(pages::templates::update),
        )
        .route(
            "/templates/:name/delete",
            post(pages::templates::delete),
        )
        // ── Logs ─────────────────────────────────────────────────────────────
        .route("/logs", get(pages::logs::list_page))
        // ── Settings ─────────────────────────────────────────────────────────
        .route("/settings", get(pages::settings::settings_page))
        .route("/settings/rotate-key", post(pages::settings::rotate_key))
        .route(
            "/settings/delete-account",
            post(pages::settings::delete_account),
        )
        // ── Static assets ────────────────────────────────────────────────────
        .nest_service("/static", ServeDir::new("frontend/static"));

    let port: u16 = std::env::var("FRONTEND_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("[frontend] listening on http://{addr}");
    tracing::info!("[frontend] backend at {}", api_base());

    axum::serve(listener, app).await?;
    Ok(())
}

fn api_base() -> String {
    std::env::var("BACKEND_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".into())
}
