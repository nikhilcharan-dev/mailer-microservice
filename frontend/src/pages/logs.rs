use axum::{
    extract::Query,
    response::{Html, IntoResponse, Response},
};
use axum_extra::extract::cookie::CookieJar;
use leptos::prelude::*;
use serde::Deserialize;
use serde_json::Value;

use crate::api;
use crate::session;
use crate::ui::{render_page, Layout};

#[derive(Deserialize)]
pub struct LogQuery {
    pub transport: Option<String>,
    pub template: Option<String>,
    pub status: Option<String>,
    pub page: Option<u64>,
}

pub async fn list_page(jar: CookieJar, Query(q): Query<LogQuery>) -> Response {
    let token = match session::token_from_jar(&jar) {
        Some(t) => t,
        None => return session::redirect_to_login().into_response(),
    };

    let page = q.page.unwrap_or(1).max(1);

    let mut qs = vec![format!("page={page}"), "limit=50".to_string()];
    if let Some(t) = &q.transport {
        qs.push(format!("transport={}", urlencoding::encode(t)));
    }
    if let Some(t) = &q.template {
        qs.push(format!("template={}", urlencoding::encode(t)));
    }
    if let Some(s) = &q.status {
        qs.push(format!("status={}", urlencoding::encode(s)));
    }
    let path = format!("/v1/logs?{}", qs.join("&"));

    let data: Value = match api::request_value(reqwest::Method::GET, &path, Some(&token), None).await
    {
        Ok(v) => v,
        Err(_) => return session::redirect_to_login().into_response(),
    };

    let logs: Vec<Value> = data
        .get("logs")
        .and_then(|x| x.as_array())
        .cloned()
        .unwrap_or_default();
    let total = data.get("total").and_then(|x| x.as_u64()).unwrap_or(0);
    let pages = data.get("pages").and_then(|x| x.as_u64()).unwrap_or(1).max(1);

    let f_transport = q.transport.clone().unwrap_or_default();
    let f_template = q.template.clone().unwrap_or_default();
    let f_status = q.status.clone().unwrap_or_default();
    let prev_page = if page > 1 { Some(page - 1) } else { None };
    let next_page = if page < pages { Some(page + 1) } else { None };

    let qs_keep = {
        let mut keep = vec![];
        if !f_transport.is_empty() {
            keep.push(format!("transport={}", urlencoding::encode(&f_transport)));
        }
        if !f_template.is_empty() {
            keep.push(format!("template={}", urlencoding::encode(&f_template)));
        }
        if !f_status.is_empty() {
            keep.push(format!("status={}", urlencoding::encode(&f_status)));
        }
        keep.join("&")
    };

    let html = render_page(move || view! {
        <Layout title="Delivery Logs" active="/logs">
            <h1>"Delivery Logs"</h1>
            <p class="subtitle">{format!("{total} total entries — filter and paginate below.")}</p>

            <div class="card">
                <form method="GET" action="/logs">
                    <div class="row-3">
                        <div>
                            <label>"Transport"</label>
                            <input type="text" name="transport" value={f_transport}/>
                        </div>
                        <div>
                            <label>"Template"</label>
                            <input type="text" name="template" value={f_template}/>
                        </div>
                        <div>
                            <label>"Status"</label>
                            <select name="status">
                                <option value="">"any"</option>
                                <option value="success" selected={f_status == "success"}>"success"</option>
                                <option value="failed" selected={f_status == "failed"}>"failed"</option>
                            </select>
                        </div>
                    </div>
                    <div class="actions">
                        <button class="btn" type="submit">"Filter"</button>
                        <a class="btn btn-ghost" href="/logs">"Clear"</a>
                    </div>
                </form>
            </div>

            <div class="card">
                {if logs.is_empty() {
                    view!{ <p style="color:var(--muted);margin:0;">"No matching logs."</p> }.into_any()
                } else {
                    view!{
                        <table>
                            <thead>
                                <tr>
                                    <th>"When"</th><th>"Transport"</th><th>"Template"</th>
                                    <th>"To"</th><th>"Status"</th><th>"Duration"</th><th>"Error"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {logs.iter().map(|l| {
                                    let sent_at = l.get("sent_at").and_then(|x| x.as_str()).unwrap_or("").to_string();
                                    let transport = l.get("transport_name").and_then(|x| x.as_str()).unwrap_or("").to_string();
                                    let template = l.get("template_name").and_then(|x| x.as_str()).unwrap_or("").to_string();
                                    let to = l.get("to").and_then(|x| x.as_str()).unwrap_or("").to_string();
                                    let status = l.get("status").and_then(|x| x.as_str()).unwrap_or("").to_string();
                                    let duration = l.get("duration_ms").and_then(|x| x.as_u64()).unwrap_or(0);
                                    let error = l.get("error").and_then(|x| x.as_str()).unwrap_or("").to_string();
                                    let tag_cls = if status == "success" { "tag success" } else { "tag danger" };
                                    view!{
                                        <tr>
                                            <td><code>{sent_at}</code></td>
                                            <td><code>{transport}</code></td>
                                            <td><code>{template}</code></td>
                                            <td>{to}</td>
                                            <td><span class={tag_cls}>{status}</span></td>
                                            <td>{format!("{duration} ms")}</td>
                                            <td style="max-width:280px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;color:var(--muted);">{error}</td>
                                        </tr>
                                    }
                                }).collect_view()}
                            </tbody>
                        </table>
                    }.into_any()
                }}

                <div class="pagination">
                    {prev_page.map(|p| {
                        let href = if qs_keep.is_empty() {
                            format!("/logs?page={p}")
                        } else {
                            format!("/logs?{qs_keep}&page={p}")
                        };
                        view!{ <a href={href}>"← Prev"</a> }
                    })}
                    <span class="current">{format!("Page {page} of {pages}")}</span>
                    {next_page.map(|p| {
                        let href = if qs_keep.is_empty() {
                            format!("/logs?page={p}")
                        } else {
                            format!("/logs?{qs_keep}&page={p}")
                        };
                        view!{ <a href={href}>"Next →"</a> }
                    })}
                </div>
            </div>
        </Layout>
    });
    Html(html).into_response()
}
