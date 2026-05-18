use axum::response::{Html, IntoResponse, Response};
use leptos::prelude::*;

use crate::ui::{render_page, PublicLayout};

const BASE: &str = "http://144.24.132.166:8080";

pub async fn docs_page() -> Response {
    let html = render_page(|| {
        view! {
            <PublicLayout title="API Reference">
                <div class="docs-wrap">
                    <aside class="docs-sidebar">
                        <div class="docs-sidebar-title">"API Reference"</div>
                        <a href="#auth">"Authentication"</a>
                        <a href="#transports">"Transports"</a>
                        <a href="#templates">"Templates"</a>
                        <a href="#send">"Send"</a>
                        <a href="#logs">"Logs"</a>
                        <a href="#account">"Account"</a>
                        <a href="#health">"Health"</a>
                    </aside>

                    <div class="docs-content">
                        <h1>"API Reference"</h1>
                        <p class="docs-intro">
                            "Base URL: "
                            <code class="base-url">{BASE}</code>
                            <br/>
                            "All endpoints are under " <code>"/v1"</code>
                            ". Authenticated endpoints require either a "
                            <strong>"JWT bearer token"</strong>
                            " (dashboard session) or an "
                            <strong>"API key"</strong>
                            " (send endpoint only)."
                        </p>

                        // ── Auth ──────────────────────────────────────────
                        <div class="docs-section" id="auth">
                            <h2>"Authentication"</h2>

                            <div class="endpoint-block">
                                <div class="endpoint-header">
                                    <span class="method-tag post">"POST"</span>
                                    <code class="ep-path">"/v1/auth/signup"</code>
                                    <span class="ep-auth">"Public"</span>
                                </div>
                                <p>"Register a new account. Returns an API key shown once."</p>
                                <div class="ep-grid">
                                    <div>
                                        <div class="ep-label">"Request body"</div>
                                        <pre class="code-block">
r#"{
  "username": "alice",
  "email": "alice@example.com",
  "password": "supersecret12chars"
}"#
                                        </pre>
                                    </div>
                                    <div>
                                        <div class="ep-label">"Response 201"</div>
                                        <pre class="code-block">
r#"{
  "username": "alice",
  "api_key": "cm_live_xxxxxxxxxxxxxxxx"
}"#
                                        </pre>
                                    </div>
                                </div>
                                <div class="ep-label">"Example"</div>
                                <pre class="code-block">{format!(
r#"curl -X POST {BASE}/v1/auth/signup \
  -H "Content-Type: application/json" \
  -d '{{"username":"alice","email":"alice@example.com","password":"supersecret12chars"}}'"#
                                )}</pre>
                            </div>

                            <div class="endpoint-block">
                                <div class="endpoint-header">
                                    <span class="method-tag post">"POST"</span>
                                    <code class="ep-path">"/v1/auth/signin"</code>
                                    <span class="ep-auth">"Public"</span>
                                </div>
                                <p>"Sign in with email or username. Returns a JWT valid for 24 hours."</p>
                                <div class="ep-grid">
                                    <div>
                                        <div class="ep-label">"Request body"</div>
                                        <pre class="code-block">
r#"{
  "identifier": "alice@example.com",
  "password": "supersecret12chars"
}"#
                                        </pre>
                                    </div>
                                    <div>
                                        <div class="ep-label">"Response 200"</div>
                                        <pre class="code-block">
r#"{
  "token": "eyJhbGciOiJIUzI1NiIsInR5..."
}"#
                                        </pre>
                                    </div>
                                </div>
                                <div class="ep-label">"Example"</div>
                                <pre class="code-block">{format!(
r#"curl -X POST {BASE}/v1/auth/signin \
  -H "Content-Type: application/json" \
  -d '{{"identifier":"alice@example.com","password":"supersecret12chars"}}'"#
                                )}</pre>
                            </div>

                            <div class="endpoint-block">
                                <div class="endpoint-header">
                                    <span class="method-tag post">"POST"</span>
                                    <code class="ep-path">"/v1/auth/apikey/rotate"</code>
                                    <span class="ep-auth">"JWT"</span>
                                </div>
                                <p>"Rotate your API key. The old key is immediately invalidated."</p>
                                <div class="ep-label">"Example"</div>
                                <pre class="code-block">{format!(
r#"curl -X POST {BASE}/v1/auth/apikey/rotate \
  -H "Authorization: Bearer $JWT""#
                                )}</pre>
                            </div>
                        </div>

                        // ── Transports ─────────────────────────────────────
                        <div class="docs-section" id="transports">
                            <h2>"Transports"</h2>
                            <p class="section-note">
                                "A transport is an SMTP configuration (host, port, TLS mode, credentials). "
                                "All transport endpoints require a JWT."
                            </p>

                            <div class="endpoint-block">
                                <div class="endpoint-header">
                                    <span class="method-tag post">"POST"</span>
                                    <code class="ep-path">"/v1/transports"</code>
                                    <span class="ep-auth">"JWT"</span>
                                </div>
                                <p>"Create a new SMTP transport."</p>
                                <div class="ep-grid">
                                    <div>
                                        <div class="ep-label">"Request body"</div>
                                        <pre class="code-block">
r#"{
  "name": "gmail",
  "smtp_host": "smtp.gmail.com",
  "smtp_port": 587,
  "tls_mode": "starttls",
  "smtp_user": "you@gmail.com",
  "smtp_pass": "app-password",
  "from_name": "Acme Alerts",
  "from_email": "you@gmail.com"
}"#
                                        </pre>
                                    </div>
                                    <div>
                                        <div class="ep-label">"tls_mode values"</div>
                                        <div class="ep-note-list">
                                            <div><code>"starttls"</code>" — port 587 (recommended)"</div>
                                            <div><code>"tls"</code>" — port 465"</div>
                                            <div><code>"none"</code>" — dev only, no encryption"</div>
                                        </div>
                                    </div>
                                </div>
                                <div class="ep-label">"Example"</div>
                                <pre class="code-block">{format!(
r#"curl -X POST {BASE}/v1/transports \
  -H "Authorization: Bearer $JWT" \
  -H "Content-Type: application/json" \
  -d '{{"name":"gmail","smtp_host":"smtp.gmail.com","smtp_port":587,"tls_mode":"starttls","smtp_user":"you@gmail.com","smtp_pass":"app-pw","from_name":"Acme","from_email":"you@gmail.com"}}'"#
                                )}</pre>
                            </div>

                            <div class="endpoint-block">
                                <div class="endpoint-header">
                                    <span class="method-tag get">"GET"</span>
                                    <code class="ep-path">"/v1/transports"</code>
                                    <span class="ep-auth">"JWT"</span>
                                </div>
                                <p>"List all transports for the authenticated user."</p>
                                <div class="ep-label">"Example"</div>
                                <pre class="code-block">{format!(
r#"curl {BASE}/v1/transports \
  -H "Authorization: Bearer $JWT""#
                                )}</pre>
                            </div>

                            <div class="endpoint-block">
                                <div class="endpoint-header">
                                    <span class="method-tag get">"GET"</span>
                                    <code class="ep-path">"/v1/transports/:name"</code>
                                    <span class="ep-auth">"JWT"</span>
                                </div>
                                <p>"Fetch a single transport by name."</p>
                            </div>

                            <div class="endpoint-block">
                                <div class="endpoint-header">
                                    <span class="method-tag put">"PUT"</span>
                                    <code class="ep-path">"/v1/transports/:name"</code>
                                    <span class="ep-auth">"JWT"</span>
                                </div>
                                <p>"Update transport fields. All fields are optional (partial update)."</p>
                            </div>

                            <div class="endpoint-block">
                                <div class="endpoint-header">
                                    <span class="method-tag delete">"DELETE"</span>
                                    <code class="ep-path">"/v1/transports/:name"</code>
                                    <span class="ep-auth">"JWT"</span>
                                </div>
                                <p>"Delete a transport."</p>
                            </div>

                            <div class="endpoint-block">
                                <div class="endpoint-header">
                                    <span class="method-tag post">"POST"</span>
                                    <code class="ep-path">"/v1/transports/:name/verify"</code>
                                    <span class="ep-auth">"JWT"</span>
                                </div>
                                <p>
                                    "Test the SMTP connection. Optionally provide "
                                    <code>"test_to"</code>
                                    " to send a real test email through the transport."
                                </p>
                                <div class="ep-grid">
                                    <div>
                                        <div class="ep-label">"Connection test only"</div>
                                        <pre class="code-block">{format!(
r#"curl -X POST {BASE}/v1/transports/gmail/verify \
  -H "Authorization: Bearer $JWT""#
                                        )}</pre>
                                    </div>
                                    <div>
                                        <div class="ep-label">"Send actual test email"</div>
                                        <pre class="code-block">{format!(
r#"curl -X POST {BASE}/v1/transports/gmail/verify \
  -H "Authorization: Bearer $JWT" \
  -H "Content-Type: application/json" \
  -d '{{"test_to":"you@example.com"}}'"#
                                        )}</pre>
                                    </div>
                                </div>
                            </div>
                        </div>

                        // ── Templates ──────────────────────────────────────
                        <div class="docs-section" id="templates">
                            <h2>"Templates"</h2>
                            <p class="section-note">
                                "Templates are HTML emails with "
                                <code>"{{variable}}"</code>
                                " tokens. Variables are extracted automatically and rendered via Handlebars at send time."
                            </p>

                            <div class="endpoint-block">
                                <div class="endpoint-header">
                                    <span class="method-tag post">"POST"</span>
                                    <code class="ep-path">"/v1/templates"</code>
                                    <span class="ep-auth">"JWT"</span>
                                </div>
                                <div class="ep-label">"Request body"</div>
                                <pre class="code-block">
r#"{
  "name": "otp",
  "subject": "Your OTP is {{otp}}",
  "body_html": "<h1>Hi {{name}}</h1><p>Your code: <b>{{otp}}</b></p>"
}"#
                                </pre>
                            </div>

                            <div class="endpoint-block">
                                <div class="endpoint-header">
                                    <span class="method-tag get">"GET"</span>
                                    <code class="ep-path">"/v1/templates"</code>
                                    <span class="ep-auth">"JWT"</span>
                                </div>
                                <p>"List all templates."</p>
                            </div>

                            <div class="endpoint-block">
                                <div class="endpoint-header">
                                    <span class="method-tag get">"GET"</span>
                                    <code class="ep-path">"/v1/templates/:name"</code>
                                    <span class="ep-auth">"JWT"</span>
                                </div>
                                <p>"Get a single template including its extracted variable list."</p>
                            </div>

                            <div class="endpoint-block">
                                <div class="endpoint-header">
                                    <span class="method-tag put">"PUT"</span>
                                    <code class="ep-path">"/v1/templates/:name"</code>
                                    <span class="ep-auth">"JWT"</span>
                                </div>
                                <p>"Update a template."</p>
                            </div>

                            <div class="endpoint-block">
                                <div class="endpoint-header">
                                    <span class="method-tag delete">"DELETE"</span>
                                    <code class="ep-path">"/v1/templates/:name"</code>
                                    <span class="ep-auth">"JWT"</span>
                                </div>
                                <p>"Delete a template."</p>
                            </div>
                        </div>

                        // ── Send ───────────────────────────────────────────
                        <div class="docs-section send-section" id="send">
                            <h2>"Send"</h2>
                            <p class="section-note">
                                "The send endpoint uses an "
                                <strong>"API key"</strong>
                                " (not JWT) so you can call it from any backend safely."
                            </p>

                            <div class="endpoint-block endpoint-featured">
                                <div class="endpoint-header">
                                    <span class="method-tag post">"POST"</span>
                                    <code class="ep-path">"/v1/send/:username/:transport/:template"</code>
                                    <span class="ep-auth">"API key"</span>
                                </div>
                                <p>
                                    "Send an email. The body is a flat JSON object whose keys map to the "
                                    "template's " <code>"{{variables}}"</code>
                                    ". The mandatory " <code>"to"</code>
                                    " field sets the recipient."
                                </p>
                                <div class="ep-grid">
                                    <div>
                                        <div class="ep-label">"Request body"</div>
                                        <pre class="code-block">
r#"{
  "to": "bob@example.com",
  "otp": "847291",
  "name": "Bob"
}"#
                                        </pre>
                                    </div>
                                    <div>
                                        <div class="ep-label">"Response 200"</div>
                                        <pre class="code-block">
r#"{
  "status": "sent",
  "message_id": "SmtpResponse { ... }"
}"#
                                        </pre>
                                    </div>
                                </div>
                                <div class="ep-label">"Example"</div>
                                <pre class="code-block">{format!(
r#"curl -X POST {BASE}/v1/send/alice/gmail/otp \
  -H "Authorization: cm_live_xxxxxxxxxxxxx" \
  -H "Content-Type: application/json" \
  -d '{{"to":"bob@example.com","otp":"847291","name":"Bob"}}'"#
                                )}</pre>
                                <div class="send-note">
                                    <strong>"Rate limited:"</strong>
                                    " subject to your account's daily send quota. "
                                    "Check remaining quota at " <code>"/v1/account"</code>"."
                                </div>
                            </div>
                        </div>

                        // ── Logs ───────────────────────────────────────────
                        <div class="docs-section" id="logs">
                            <h2>"Logs"</h2>

                            <div class="endpoint-block">
                                <div class="endpoint-header">
                                    <span class="method-tag get">"GET"</span>
                                    <code class="ep-path">"/v1/logs"</code>
                                    <span class="ep-auth">"JWT"</span>
                                </div>
                                <p>"Paginated delivery log with optional filters."</p>
                                <div class="ep-label">"Query parameters"</div>
                                <div class="ep-note-list">
                                    <div><code>"transport"</code>" — filter by transport name"</div>
                                    <div><code>"template"</code>" — filter by template name"</div>
                                    <div><code>"status"</code>" — " <code>"success"</code> " | " <code>"failed"</code></div>
                                    <div><code>"page"</code>" — page number (default 1)"</div>
                                    <div><code>"limit"</code>" — items per page (default 50)"</div>
                                </div>
                                <div class="ep-label">"Example"</div>
                                <pre class="code-block">{format!(
r#"curl "{BASE}/v1/logs?transport=gmail&status=failed&page=1" \
  -H "Authorization: Bearer $JWT""#
                                )}</pre>
                            </div>
                        </div>

                        // ── Account ────────────────────────────────────────
                        <div class="docs-section" id="account">
                            <h2>"Account"</h2>

                            <div class="endpoint-block">
                                <div class="endpoint-header">
                                    <span class="method-tag get">"GET"</span>
                                    <code class="ep-path">"/v1/account"</code>
                                    <span class="ep-auth">"JWT"</span>
                                </div>
                                <p>"Get account info: username, email, daily quota, sent today, transport and template counts."</p>
                                <div class="ep-label">"Response 200"</div>
                                <pre class="code-block">
r#"{
  "username": "alice",
  "email": "alice@example.com",
  "daily_limit": 500,
  "sent_today": 12,
  "transport_count": 2,
  "template_count": 4,
  "created_at": "2025-01-01T00:00:00Z"
}"#
                                </pre>
                            </div>

                            <div class="endpoint-block">
                                <div class="endpoint-header">
                                    <span class="method-tag delete">"DELETE"</span>
                                    <code class="ep-path">"/v1/account"</code>
                                    <span class="ep-auth">"JWT"</span>
                                </div>
                                <p>"Permanently delete the account and all associated data. Requires password confirmation."</p>
                                <div class="ep-label">"Request body"</div>
                                <pre class="code-block">r#"{ "password": "current-password" }"#</pre>
                            </div>
                        </div>

                        // ── Health ─────────────────────────────────────────
                        <div class="docs-section" id="health">
                            <h2>"Health"</h2>

                            <div class="endpoint-block">
                                <div class="endpoint-header">
                                    <span class="method-tag get">"GET"</span>
                                    <code class="ep-path">"/health"</code>
                                    <span class="ep-auth">"None"</span>
                                </div>
                                <p>"Returns " <code>"200 OK"</code> " when the service is alive."</p>
                            </div>

                            <div class="endpoint-block">
                                <div class="endpoint-header">
                                    <span class="method-tag get">"GET"</span>
                                    <code class="ep-path">"/ready"</code>
                                    <span class="ep-auth">"None"</span>
                                </div>
                                <p>"Returns " <code>"200 OK"</code> " when MongoDB and Redis connections are live."</p>
                            </div>
                        </div>

                    </div>
                </div>
            </PublicLayout>
        }
    });
    Html(html).into_response()
}
