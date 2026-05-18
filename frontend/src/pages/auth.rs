use axum::{
    extract::Query,
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use axum_extra::extract::cookie::CookieJar;
use leptos::prelude::*;
use serde::Deserialize;
use serde_json::json;

use crate::api;
use crate::session;
use crate::ui::{render_page, AuthLayout, Flash};

#[derive(Deserialize)]
pub struct ErrParam {
    pub err: Option<String>,
}

fn err_msg(code: &str) -> &'static str {
    match code {
        "invalid_credentials" => "Email or password is incorrect.",
        "username_taken" => "That username is already taken.",
        "email_taken" => "That email is already registered.",
        "invalid_email" => "Email format is invalid.",
        "invalid_username" => "Username must be 3-32 chars, lowercase, alphanumeric (+ _ or -).",
        "weak_password" => "Password must be at least 12 characters.",
        "rate_limited" => "Too many attempts; please wait and try again.",
        "backend_unreachable" => "Backend is unreachable. Try again shortly.",
        _ => "Something went wrong; try again.",
    }
}

// ── GET /login ───────────────────────────────────────────────────────────────
pub async fn login_page(Query(q): Query<ErrParam>) -> Html<String> {
    let flash = q.err.clone();
    let html = render_page(move || {
        view! {
            <AuthLayout title="Sign in">
                <div class="card">
                    <h1>"Sign in"</h1>
                    <p class="subtitle">"cloudMailer dashboard"</p>
                    {flash.as_ref().map(|c| view!{ <Flash kind="error" msg={err_msg(c).to_string()}/> })}
                    <form method="POST" action="/login">
                        <label>"Email or username"</label>
                        <input type="text" name="identifier" required autofocus/>
                        <label>"Password"</label>
                        <input type="password" name="password" required/>
                        <div class="actions">
                            <button class="btn" type="submit">"Sign in"</button>
                        </div>
                    </form>
                    <div class="switch">
                        "No account? "<a href="/signup">"Create one"</a>
                    </div>
                </div>
            </AuthLayout>
        }
    });
    Html(html)
}

#[derive(Deserialize)]
pub struct LoginForm {
    pub identifier: String,
    pub password: String,
}

// ── POST /login ──────────────────────────────────────────────────────────────
pub async fn login_submit(jar: CookieJar, Form(f): Form<LoginForm>) -> Response {
    let body = json!({ "identifier": f.identifier, "password": f.password });
    match api::request_value(reqwest::Method::POST, "/v1/auth/signin", None, Some(body)).await {
        Ok(v) => {
            let token = v.get("token").and_then(|x| x.as_str()).unwrap_or("").to_string();
            if token.is_empty() {
                return Redirect::to("/login?err=error").into_response();
            }
            let jar = session::set_token(jar, token);
            (jar, Redirect::to("/dashboard")).into_response()
        }
        Err(e) => Redirect::to(&format!("/login?err={}", e.code)).into_response(),
    }
}

// ── GET /signup ──────────────────────────────────────────────────────────────
pub async fn signup_page(Query(q): Query<ErrParam>) -> Html<String> {
    let flash = q.err.clone();
    let html = render_page(move || {
        view! {
            <AuthLayout title="Create account">
                <div class="card">
                    <h1>"Create account"</h1>
                    <p class="subtitle">"Start sending in minutes"</p>
                    {flash.as_ref().map(|c| view!{ <Flash kind="error" msg={err_msg(c).to_string()}/> })}
                    <form method="POST" action="/signup">
                        <label>"Username"</label>
                        <input type="text" name="username" required minlength="3" maxlength="32" autofocus/>
                        <label>"Email"</label>
                        <input type="email" name="email" required/>
                        <label>"Password"</label>
                        <input type="password" name="password" required minlength="12"/>
                        <div class="actions">
                            <button class="btn" type="submit">"Create account"</button>
                        </div>
                    </form>
                    <div class="switch">
                        "Already registered? "<a href="/login">"Sign in"</a>
                    </div>
                </div>
            </AuthLayout>
        }
    });
    Html(html)
}

#[derive(Deserialize)]
pub struct SignupForm {
    pub username: String,
    pub email: String,
    pub password: String,
}

// ── POST /signup ─────────────────────────────────────────────────────────────
pub async fn signup_submit(jar: CookieJar, Form(f): Form<SignupForm>) -> Response {
    let body = json!({
        "username": f.username.clone(),
        "email": f.email.clone(),
        "password": f.password.clone(),
    });
    match api::request_value(reqwest::Method::POST, "/v1/auth/signup", None, Some(body)).await {
        Ok(v) => {
            let api_key = v
                .get("api_key")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            // Now sign in to obtain a JWT
            let body2 = json!({ "identifier": f.email, "password": f.password });
            match api::request_value(reqwest::Method::POST, "/v1/auth/signin", None, Some(body2)).await {
                Ok(s) => {
                    let token = s
                        .get("token")
                        .and_then(|x| x.as_str())
                        .unwrap_or("")
                        .to_string();
                    let jar = session::set_token(jar, token);
                    (
                        jar,
                        Redirect::to(&format!(
                            "/settings?just_created=1&key={}",
                            urlencoding::encode(&api_key)
                        )),
                    )
                        .into_response()
                }
                Err(_) => Redirect::to("/login").into_response(),
            }
        }
        Err(e) => Redirect::to(&format!("/signup?err={}", e.code)).into_response(),
    }
}

// ── POST /logout ─────────────────────────────────────────────────────────────
pub async fn logout(jar: CookieJar) -> Response {
    let jar = session::clear_token(jar);
    (jar, Redirect::to("/login")).into_response()
}
