use axum::{
    extract::Query,
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use axum_extra::extract::cookie::CookieJar;
use leptos::prelude::*;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::api;
use crate::session;
use crate::ui::{render_page, Flash, Layout};

#[derive(Deserialize)]
pub struct SettingsQuery {
    pub err: Option<String>,
    pub ok: Option<String>,
    pub key: Option<String>,
    pub just_created: Option<String>,
}

pub async fn settings_page(jar: CookieJar, Query(q): Query<SettingsQuery>) -> Response {
    let token = match session::token_from_jar(&jar) {
        Some(t) => t,
        None => return session::redirect_to_login().into_response(),
    };

    let account: Value =
        match api::request_value(reqwest::Method::GET, "/v1/account", Some(&token), None).await {
            Ok(v) => v,
            Err(_) => return session::redirect_to_login().into_response(),
        };

    let username = account
        .get("username")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let email = account
        .get("email")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let daily_limit = account
        .get("daily_limit")
        .and_then(|x| x.as_u64())
        .unwrap_or(0);
    let created_at = account
        .get("created_at")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();

    let err = q.err.clone();
    let ok = q.ok.clone();
    let shown_key = q.key.clone();
    let just_created = q.just_created.is_some();

    let html = render_page(move || view! {
        <Layout title="Settings" active="/settings">
            <h1>"Account Settings"</h1>
            <p class="subtitle">"Manage your account, view limits, rotate your API key."</p>
            {err.map(|e| view!{ <Flash kind="error" msg={e}/> })}
            {ok.map(|o| view!{ <Flash kind="success" msg={o}/> })}

            {shown_key.map(|k| {
                let msg = if just_created {
                    "Account created. Store this API key — it will not be shown again."
                } else {
                    "Here is your new API key. Store it now — it will not be shown again."
                };
                view!{
                    <div class="card" style="border-color:var(--accent);">
                        <h2>"⚠ Your API key"</h2>
                        <p style="color:var(--muted); margin-top:0;">{msg}</p>
                        <div class="api-key mono">{k}</div>
                    </div>
                }
            })}

            <div class="card">
                <h2>"Account"</h2>
                <div class="row">
                    <div><label>"Username"</label><div class="mono">{username}</div></div>
                    <div><label>"Email"</label><div class="mono">{email}</div></div>
                </div>
                <div class="row">
                    <div><label>"Daily send limit"</label><div class="mono">{daily_limit.to_string()}</div></div>
                    <div><label>"Member since"</label><div class="mono">{created_at}</div></div>
                </div>
            </div>

            <div class="card">
                <h2>"API key"</h2>
                <p style="color:var(--muted); margin-top:0;">
                    "Rotating regenerates a new API key. The previous key stops working within 15 minutes (cache TTL)."
                </p>
                <form method="POST" action="/settings/rotate-key"
                      onsubmit="return confirm('Rotate API key? This will invalidate the current key.')">
                    <div class="actions">
                        <button class="btn btn-danger" type="submit">"Rotate API key"</button>
                    </div>
                </form>
            </div>

            <div class="card" style="border-color:var(--danger);">
                <h2 style="color:var(--danger);">"Delete account"</h2>
                <p style="color:var(--muted); margin-top:0;">
                    "Permanently delete your account, all transports, and all templates.
                    Delivery logs are retained for 90 days for abuse investigation, then auto-purged."
                </p>
                <form method="POST" action="/settings/delete-account"
                      onsubmit="return confirm('Really delete your account? This cannot be undone.')">
                    <label>"Confirm with your password"</label>
                    <input type="password" name="password" required/>
                    <div class="actions">
                        <button class="btn btn-danger" type="submit">"Delete my account"</button>
                    </div>
                </form>
            </div>
        </Layout>
    });
    Html(html).into_response()
}

pub async fn rotate_key(jar: CookieJar) -> Response {
    let token = match session::token_from_jar(&jar) {
        Some(t) => t,
        None => return session::redirect_to_login().into_response(),
    };
    match api::request_value(
        reqwest::Method::POST,
        "/v1/auth/apikey/rotate",
        Some(&token),
        None,
    )
    .await
    {
        Ok(v) => {
            let key = v.get("api_key").and_then(|x| x.as_str()).unwrap_or("");
            Redirect::to(&format!("/settings?key={}", urlencoding::encode(key))).into_response()
        }
        Err(e) => Redirect::to(&format!("/settings?err={}", urlencoding::encode(&e.message)))
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct DeleteForm {
    pub password: String,
}

pub async fn delete_account(jar: CookieJar, Form(f): Form<DeleteForm>) -> Response {
    let token = match session::token_from_jar(&jar) {
        Some(t) => t,
        None => return session::redirect_to_login().into_response(),
    };
    let body = json!({ "password": f.password });
    match api::request_value(reqwest::Method::DELETE, "/v1/account", Some(&token), Some(body)).await
    {
        Ok(_) => {
            let jar = session::clear_token(jar);
            (jar, Redirect::to("/login")).into_response()
        }
        Err(e) => Redirect::to(&format!("/settings?err={}", urlencoding::encode(&e.message)))
            .into_response(),
    }
}
