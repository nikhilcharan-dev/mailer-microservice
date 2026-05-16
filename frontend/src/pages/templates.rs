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

    let list: Value =
        match api::request_value(reqwest::Method::GET, "/v1/templates", Some(&token), None).await {
            Ok(v) => v,
            Err(_) => return session::redirect_to_login().into_response(),
        };
    let items: Vec<Value> = list.as_array().cloned().unwrap_or_default();
    let err = q.err.clone();
    let ok = q.ok.clone();

    let html = render_page(move || view! {
        <Layout title="Templates" active="/templates">
            <h1>"HTML Templates"</h1>
            <p class="subtitle">"Upload raw HTML with "<code>"{{variable}}"</code>" tokens. Variables are extracted and validated on send."</p>
            {err.map(|e| view!{ <Flash kind="error" msg={e}/> })}
            {ok.map(|o| view!{ <Flash kind="success" msg={o}/> })}

            <div class="card">
                <h2>"Your templates"</h2>
                {if items.is_empty() {
                    view!{ <p style="color:var(--muted);margin:0;">"No templates yet — add one below."</p> }.into_any()
                } else {
                    view!{
                        <table>
                            <thead>
                                <tr><th>"Name"</th><th>"Subject"</th><th>"Variables"</th><th></th></tr>
                            </thead>
                            <tbody>
                                {items.iter().map(|t| {
                                    let name = t.get("name").and_then(|x| x.as_str()).unwrap_or("").to_string();
                                    let subject = t.get("subject").and_then(|x| x.as_str()).unwrap_or("").to_string();
                                    let variables: Vec<String> = t.get("variables")
                                        .and_then(|x| x.as_array())
                                        .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                                        .unwrap_or_default();
                                    let name_v = name.clone();
                                    let name_d = name.clone();
                                    view!{
                                        <tr>
                                            <td><a href={format!("/templates/{name}")}><code>{name.clone()}</code></a></td>
                                            <td style="max-width:280px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;">{subject}</td>
                                            <td>
                                                <div class="var-badges">
                                                    {variables.iter().map(|v| view!{ <code>{format!("{{{{{v}}}}}")}</code> }).collect_view()}
                                                </div>
                                            </td>
                                            <td style="text-align:right;">
                                                <a href={format!("/templates/{name_v}")} class="btn btn-ghost">"Edit"</a>
                                                " "
                                                <form method="POST" action={format!("/templates/{name_d}/delete")} style="display:inline"
                                                      onsubmit="return confirm('Delete this template?')">
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
                <h2>"Add a template"</h2>
                <form method="POST" action="/templates">
                    <div class="row">
                        <div>
                            <label>"Name (URL-safe)"</label>
                            <input type="text" name="name" placeholder="otp" required/>
                        </div>
                        <div>
                            <label>"Default subject (may use {{variables}})"</label>
                            <input type="text" name="subject" placeholder="Your code is {{otp}}" required/>
                        </div>
                    </div>
                    <label>"HTML body"</label>
                    <textarea name="html" required
                              placeholder="<p>Hi {{name}}, your code is {{otp}}.</p>"></textarea>
                    <div class="actions">
                        <button class="btn" type="submit">"Add template"</button>
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
    pub subject: String,
    pub html: String,
}

pub async fn create(jar: CookieJar, Form(f): Form<CreateForm>) -> Response {
    let token = match session::token_from_jar(&jar) {
        Some(t) => t,
        None => return session::redirect_to_login().into_response(),
    };
    let body = json!({ "name": f.name, "subject": f.subject, "html": f.html });
    match api::request_value(reqwest::Method::POST, "/v1/templates", Some(&token), Some(body)).await {
        Ok(_) => Redirect::to("/templates?ok=Template+added").into_response(),
        Err(e) => Redirect::to(&format!("/templates?err={}", urlencoding::encode(&e.message)))
            .into_response(),
    }
}

pub async fn detail_page(jar: CookieJar, Path(name): Path<String>, Query(q): Query<Flash_>) -> Response {
    let token = match session::token_from_jar(&jar) {
        Some(t) => t,
        None => return session::redirect_to_login().into_response(),
    };
    let path = format!("/v1/templates/{}", urlencoding::encode(&name));
    let t: Value = match api::request_value(reqwest::Method::GET, &path, Some(&token), None).await {
        Ok(v) => v,
        Err(_) => return Redirect::to("/templates?err=Template+not+found").into_response(),
    };

    let subject = t.get("subject").and_then(|x| x.as_str()).unwrap_or("").to_string();
    let html_body = t.get("html").and_then(|x| x.as_str()).unwrap_or("").to_string();
    let variables: Vec<String> = t
        .get("variables")
        .and_then(|x| x.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let err = q.err.clone();
    let ok = q.ok.clone();
    let name_clone = name.clone();

    let html = render_page(move || view! {
        <Layout title="Edit template" active="/templates">
            <h1>{format!("Template: {name_clone}")}</h1>
            <p class="subtitle">"Edit the subject or HTML body. Variables are re-extracted on save."</p>
            {err.map(|e| view!{ <Flash kind="error" msg={e}/> })}
            {ok.map(|o| view!{ <Flash kind="success" msg={o}/> })}

            <div class="card">
                <h2>"Variables in this template"</h2>
                {if variables.is_empty() {
                    view!{ <p style="color:var(--muted);margin:0;">"No variables detected."</p> }.into_any()
                } else {
                    view!{
                        <div class="var-badges">
                            {variables.iter().map(|v| view!{ <code>{format!("{{{{{v}}}}}")}</code> }).collect_view()}
                        </div>
                    }.into_any()
                }}
            </div>

            <div class="card">
                <h2>"Edit"</h2>
                <form method="POST" action={format!("/templates/{name_clone}")}>
                    <label>"Subject"</label>
                    <input type="text" name="subject" value={subject} required/>
                    <label>"HTML body"</label>
                    <textarea name="html" required>{html_body}</textarea>
                    <div class="actions">
                        <button class="btn" type="submit">"Save"</button>
                        <a class="btn btn-ghost" href="/templates">"Back"</a>
                    </div>
                </form>
            </div>
        </Layout>
    });
    Html(html).into_response()
}

#[derive(Deserialize)]
pub struct UpdateForm {
    pub subject: String,
    pub html: String,
}

pub async fn update(jar: CookieJar, Path(name): Path<String>, Form(f): Form<UpdateForm>) -> Response {
    let token = match session::token_from_jar(&jar) {
        Some(t) => t,
        None => return session::redirect_to_login().into_response(),
    };
    let body = json!({ "subject": f.subject, "html": f.html });
    let path = format!("/v1/templates/{}", urlencoding::encode(&name));
    match api::request_value(reqwest::Method::PUT, &path, Some(&token), Some(body)).await {
        Ok(_) => Redirect::to(&format!(
            "/templates/{}?ok=Saved",
            urlencoding::encode(&name)
        ))
        .into_response(),
        Err(e) => Redirect::to(&format!(
            "/templates/{}?err={}",
            urlencoding::encode(&name),
            urlencoding::encode(&e.message)
        ))
        .into_response(),
    }
}

pub async fn delete(jar: CookieJar, Path(name): Path<String>) -> Response {
    let token = match session::token_from_jar(&jar) {
        Some(t) => t,
        None => return session::redirect_to_login().into_response(),
    };
    let path = format!("/v1/templates/{}", urlencoding::encode(&name));
    match api::request_value(reqwest::Method::DELETE, &path, Some(&token), None).await {
        Ok(_) => Redirect::to("/templates?ok=Deleted").into_response(),
        Err(e) => Redirect::to(&format!("/templates?err={}", urlencoding::encode(&e.message)))
            .into_response(),
    }
}
