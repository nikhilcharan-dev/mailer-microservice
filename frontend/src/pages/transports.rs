use axum::{
    extract::{Path, Query},
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
pub struct Flash_ {
    pub err: Option<String>,
    pub ok: Option<String>,
}

pub async fn list_page(jar: CookieJar, Query(q): Query<Flash_>) -> Response {
    let token = match session::token_from_jar(&jar) {
        Some(t) => t,
        None => return session::redirect_to_login().into_response(),
    };

    let list: Value = match api::request_value(
        reqwest::Method::GET,
        "/v1/transports",
        Some(&token),
        None,
    )
    .await
    {
        Ok(v) => v,
        Err(_) => return session::redirect_to_login().into_response(),
    };

    let items: Vec<Value> = list.as_array().cloned().unwrap_or_default();
    let err = q.err.clone();
    let ok = q.ok.clone();

    let html = render_page(move || view! {
        <Layout title="Transports" active="/transports">
            <h1>"SMTP Transports"</h1>
            <p class="subtitle">"Configure one or more SMTP backends to send through."</p>
            {err.map(|e| view!{ <Flash kind="error" msg={e}/> })}
            {ok.map(|o| view!{ <Flash kind="success" msg={o}/> })}

            <div class="card">
                <h2>"Your transports"</h2>
                {if items.is_empty() {
                    view!{ <p style="color:var(--muted);margin:0;">"No transports yet — add one below."</p> }.into_any()
                } else {
                    view!{
                        <table>
                            <thead>
                                <tr>
                                    <th>"Name"</th><th>"Host"</th><th>"Port"</th>
                                    <th>"TLS"</th><th>"From"</th><th></th>
                                </tr>
                            </thead>
                            <tbody>
                                {items.iter().map(|t| {
                                    let name = t.get("name").and_then(|x| x.as_str()).unwrap_or("").to_string();
                                    let host = t.get("smtp_host").and_then(|x| x.as_str()).unwrap_or("").to_string();
                                    let port = t.get("smtp_port").and_then(|x| x.as_u64()).unwrap_or(0);
                                    let tls = t.get("tls_mode").and_then(|x| x.as_str()).unwrap_or("").to_string();
                                    let from = t.get("from_email").and_then(|x| x.as_str()).unwrap_or("").to_string();
                                    let name_v = name.clone();
                                    let name_d = name.clone();
                                    view!{
                                        <tr>
                                            <td><code>{name}</code></td>
                                            <td>{host}</td>
                                            <td>{port.to_string()}</td>
                                            <td><span class="tag muted">{tls}</span></td>
                                            <td>{from}</td>
                                            <td style="text-align:right;display:flex;gap:6px;justify-content:flex-end;">
                                                <form method="POST" action={format!("/transports/{name_v}/verify")} style="display:inline">
                                                    <button class="btn btn-ghost" type="submit">"Test"</button>
                                                </form>
                                                <form method="POST" action={format!("/transports/{name_d}/delete")} style="display:inline"
                                                      onsubmit="return confirm('Delete this transport?')">
                                                    <button class="btn btn-danger" type="submit">"Delete"</button>
                                                </form>
                                            </td>
                                        </tr>
                                    }
                                }).collect_view()}
                            </tbody>
                        </table>
                    }.into_any()
                }}
            </div>

            <div class="card">
                <h2>"Add a transport"</h2>
                <form method="POST" action="/transports">
                    <div class="row">
                        <div>
                            <label>"Name (URL-safe)"</label>
                            <input type="text" name="name" placeholder="gmail" required/>
                        </div>
                        <div>
                            <label>"TLS mode"</label>
                            <select name="tls_mode">
                                <option value="starttls">"starttls (port 587)"</option>
                                <option value="tls">"tls (port 465)"</option>
                                <option value="none">"none (dev only)"</option>
                            </select>
                        </div>
                    </div>
                    <div class="row">
                        <div>
                            <label>"SMTP host"</label>
                            <input type="text" name="smtp_host" placeholder="smtp.gmail.com" required/>
                        </div>
                        <div>
                            <label>"SMTP port"</label>
                            <input type="number" name="smtp_port" value="587" required/>
                        </div>
                    </div>
                    <div class="row">
                        <div>
                            <label>"SMTP user"</label>
                            <input type="text" name="smtp_user" placeholder="you@gmail.com" required/>
                        </div>
                        <div>
                            <label>"SMTP password"</label>
                            <input type="password" name="smtp_pass" required/>
                        </div>
                    </div>
                    <div class="row">
                        <div>
                            <label>"From name"</label>
                            <input type="text" name="from_name" placeholder="Acme Alerts" required/>
                        </div>
                        <div>
                            <label>"From email"</label>
                            <input type="email" name="from_email" required/>
                        </div>
                    </div>
                    <div class="actions">
                        <button class="btn" type="submit">"Add transport"</button>
                    </div>
                </form>
            </div>
        </Layout>
    });
    Html(html).into_response()
}

#[derive(Deserialize)]
pub struct CreateForm {
    pub name: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub tls_mode: String,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub from_name: String,
    pub from_email: String,
}

pub async fn create(jar: CookieJar, Form(f): Form<CreateForm>) -> Response {
    let token = match session::token_from_jar(&jar) {
        Some(t) => t,
        None => return session::redirect_to_login().into_response(),
    };
    let body = json!({
        "name": f.name, "smtp_host": f.smtp_host, "smtp_port": f.smtp_port,
        "tls_mode": f.tls_mode, "smtp_user": f.smtp_user, "smtp_pass": f.smtp_pass,
        "from_name": f.from_name, "from_email": f.from_email,
    });
    match api::request_value(reqwest::Method::POST, "/v1/transports", Some(&token), Some(body)).await {
        Ok(_) => Redirect::to("/transports?ok=Transport+added").into_response(),
        Err(e) => Redirect::to(&format!("/transports?err={}", urlencoding::encode(&e.message)))
            .into_response(),
    }
}

pub async fn delete(jar: CookieJar, Path(name): Path<String>) -> Response {
    let token = match session::token_from_jar(&jar) {
        Some(t) => t,
        None => return session::redirect_to_login().into_response(),
    };
    let path = format!("/v1/transports/{}", urlencoding::encode(&name));
    match api::request_value(reqwest::Method::DELETE, &path, Some(&token), None).await {
        Ok(_) => Redirect::to("/transports?ok=Deleted").into_response(),
        Err(e) => Redirect::to(&format!("/transports?err={}", urlencoding::encode(&e.message)))
            .into_response(),
    }
}

pub async fn verify(jar: CookieJar, Path(name): Path<String>) -> Response {
    let token = match session::token_from_jar(&jar) {
        Some(t) => t,
        None => return session::redirect_to_login().into_response(),
    };
    let path = format!("/v1/transports/{}/verify", urlencoding::encode(&name));
    match api::request_value(reqwest::Method::POST, &path, Some(&token), None).await {
        Ok(v) => {
            let ok = v.get("ok").and_then(|x| x.as_bool()).unwrap_or(false);
            if ok {
                Redirect::to("/transports?ok=Connection+OK").into_response()
            } else {
                let msg = v
                    .get("error")
                    .and_then(|x| x.as_str())
                    .unwrap_or("verify failed");
                Redirect::to(&format!("/transports?err={}", urlencoding::encode(msg)))
                    .into_response()
            }
        }
        Err(e) => Redirect::to(&format!("/transports?err={}", urlencoding::encode(&e.message)))
            .into_response(),
    }
}
