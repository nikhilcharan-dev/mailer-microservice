use axum::response::{Html, IntoResponse, Response};
use axum_extra::extract::cookie::CookieJar;
use leptos::prelude::*;
use serde_json::Value;

use crate::api;
use crate::session;
use crate::ui::{render_page, Layout};

pub async fn dashboard(jar: CookieJar) -> Response {
    let token = match session::token_from_jar(&jar) {
        Some(t) => t,
        None => return session::redirect_to_login().into_response(),
    };

    let account: Value = match api::request_value(
        reqwest::Method::GET,
        "/v1/account",
        Some(&token),
        None,
    )
    .await
    {
        Ok(v) => v,
        Err(_) => return session::redirect_to_login().into_response(),
    };

    let username = account
        .get("username")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let sent_today = account.get("sent_today").and_then(|x| x.as_u64()).unwrap_or(0);
    let daily_limit = account.get("daily_limit").and_then(|x| x.as_u64()).unwrap_or(0);
    let transport_count = account
        .get("transport_count")
        .and_then(|x| x.as_u64())
        .unwrap_or(0);
    let template_count = account
        .get("template_count")
        .and_then(|x| x.as_u64())
        .unwrap_or(0);
    let remaining = daily_limit.saturating_sub(sent_today);
    let usage_pct = if daily_limit > 0 {
        (sent_today * 100) / daily_limit
    } else {
        0
    };

    let html = render_page(move || view! {
        <Layout title="Overview" active="/">
            <h1>{format!("Welcome back, {username}")}</h1>
            <p class="subtitle">"At a glance: today's send activity and your account quotas."</p>
            <div class="stats">
                <div class="stat">
                    <div class="label">"Sent today"</div>
                    <div class="value">{sent_today.to_string()}</div>
                </div>
                <div class="stat">
                    <div class="label">"Daily quota"</div>
                    <div class="value">{format!("{}/{}", sent_today, daily_limit)}</div>
                </div>
                <div class="stat">
                    <div class="label">"Remaining"</div>
                    <div class="value success">{remaining.to_string()}</div>
                </div>
                <div class="stat">
                    <div class="label">"Quota used"</div>
                    <div class="value">{format!("{usage_pct}%")}</div>
                </div>
                <div class="stat">
                    <div class="label">"Transports"</div>
                    <div class="value">{transport_count.to_string()}</div>
                </div>
                <div class="stat">
                    <div class="label">"Templates"</div>
                    <div class="value">{template_count.to_string()}</div>
                </div>
            </div>
            <div class="card">
                <h2>"Quick start"</h2>
                <ol style="margin: 0; padding-left: 22px; color: var(--muted); line-height: 1.9;">
                    <li>"Add an SMTP transport on the "<a href="/transports">"Transports"</a>" page."</li>
                    <li>"Upload an HTML template with "<code>"{{variable}}"</code>" tokens on the "<a href="/templates">"Templates"</a>" page."</li>
                    <li>"Grab your API key from "<a href="/settings">"Settings"</a>"."</li>
                    <li>"POST to "<code>"/v1/send/:username/:transport/:template"</code>" with your variables — done."</li>
                </ol>
            </div>
        </Layout>
    });
    Html(html).into_response()
}
