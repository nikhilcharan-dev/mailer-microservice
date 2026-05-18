use axum::response::{Html, IntoResponse, Response};
use leptos::prelude::*;

use crate::ui::{render_page, PublicLayout};

pub async fn home() -> Response {
    let html = render_page(|| {
        view! {
            <PublicLayout title="Your SMTP. Your Delivery. Zero Lock-in.">

                // ── Hero ────────────────────────────────────────────────────
                <section class="hero">
                    <div class="hero-inner">
                        <div class="hero-eyebrow">"Open-source · Self-hostable · Zero lock-in"</div>
                        <h1 class="hero-title">
                            "Your SMTP." <br/>
                            "Your delivery." <br/>
                            <span class="hero-accent">"Zero lock-in."</span>
                        </h1>
                        <p class="hero-sub">
                            "cloudMailer is a multi-tenant email relay. Connect your own Gmail, Outlook, "
                            "SendGrid SMTP, or any provider and send through one clean REST API — "
                            "you own every credential, every domain, every byte of data."
                        </p>
                        <div class="hero-cta">
                            <a href="/signup" class="btn-hero">"Get started free"</a>
                            <a href="/docs" class="btn-hero-ghost">"View API docs →"</a>
                        </div>
                        <div class="hero-code">
                            <div class="code-label">"Send an email in one request"</div>
                            <pre class="code-preview">{r#"curl -X POST http://144.24.132.166:8080/v1/send/alice/gmail/otp \
  -H "Authorization: cm_live_xxxxxxxxxxxxx" \
  -H "Content-Type: application/json" \
  -d '{"to":"bob@example.com","otp":"847291"}'"#}</pre>
                        </div>
                    </div>
                </section>

                // ── Why Rust ────────────────────────────────────────────────
                <section class="pub-section">
                    <div class="section-inner">
                        <div class="section-tag">"Built with 🦀 Rust"</div>
                        <h2 class="section-title">"Performance and safety, by default"</h2>
                        <p class="section-sub">
                            "No garbage collector. No runtime overhead. No memory-safety CVEs. "
                            "Every send is deterministic."
                        </p>
                        <div class="feature-grid">
                            <div class="feature-card">
                                <div class="feature-icon">"⚡"</div>
                                <h3>"Sub-millisecond overhead"</h3>
                                <p>
                                    "Rust's async runtime (Tokio) has zero GC pauses. "
                                    "Every email relay is deterministic — no latency spikes under load, "
                                    "no stop-the-world surprises."
                                </p>
                            </div>
                            <div class="feature-card">
                                <div class="feature-icon">"🔒"</div>
                                <h3>"Memory-safe by construction"</h3>
                                <p>
                                    "The borrow checker eliminates entire classes of bugs at compile time. "
                                    "No buffer overflows in your mail pipeline — ever. "
                                    "Safety is enforced, not promised."
                                </p>
                            </div>
                            <div class="feature-card">
                                <div class="feature-icon">"📦"</div>
                                <h3>"Tiny footprint"</h3>
                                <p>
                                    "The entire binary is ~10 MB. Idle RAM sits under 15 MB. "
                                    "Ship it on the smallest VM you have — no JVM, no Node runtime, "
                                    "no Python interpreter."
                                </p>
                            </div>
                            <div class="feature-card">
                                <div class="feature-icon">"🛡"</div>
                                <h3>"Crypto-first"</h3>
                                <p>
                                    "SMTP credentials encrypted at rest with AES-256-GCM. "
                                    "Passwords hashed with Argon2id. "
                                    "JWT-authenticated API. Security is a first-class citizen."
                                </p>
                            </div>
                            <div class="feature-card">
                                <div class="feature-icon">"🚀"</div>
                                <h3>"Fearless concurrency"</h3>
                                <p>
                                    "Rust's ownership model guarantees data-race freedom at compile time. "
                                    "Handle thousands of concurrent sends safely, "
                                    "with Redis-backed rate limiting built in."
                                </p>
                            </div>
                            <div class="feature-card">
                                <div class="feature-icon">"🐳"</div>
                                <h3>"Docker-ready in 60 seconds"</h3>
                                <p>
                                    "Multi-stage Dockerfile produces a minimal Debian image. "
                                    "One " <code>"docker compose up"</code> " and you're live. "
                                    "Bring your own MongoDB Atlas + Redis Cloud."
                                </p>
                            </div>
                        </div>
                    </div>
                </section>

                // ── How it works ────────────────────────────────────────────
                <section class="pub-section alt">
                    <div class="section-inner">
                        <div class="section-tag">"How it works"</div>
                        <h2 class="section-title">"Send in three steps"</h2>
                        <div class="steps">
                            <div class="step">
                                <div class="step-num">"1"</div>
                                <div class="step-body">
                                    <h3>"Connect your SMTP"</h3>
                                    <p>
                                        "Register and add your Gmail, Outlook, or any SMTP provider credentials. "
                                        "cloudMailer encrypts them at rest with AES-256-GCM — "
                                        "we never see your passwords in plaintext."
                                    </p>
                                </div>
                            </div>
                            <div class="step">
                                <div class="step-num">"2"</div>
                                <div class="step-body">
                                    <h3>"Upload a template"</h3>
                                    <p>
                                        "Write an HTML email with "
                                        <code>"{{variable}}"</code>
                                        " tokens. cloudMailer renders them at send time using "
                                        "Handlebars — open, portable, no proprietary DSL."
                                    </p>
                                </div>
                            </div>
                            <div class="step">
                                <div class="step-num">"3"</div>
                                <div class="step-body">
                                    <h3>"Call /v1/send"</h3>
                                    <p>
                                        "POST your API key with the variable values. The email goes out "
                                        "through YOUR transport. Delivery logs are captured automatically "
                                        "with status, duration, and error details."
                                    </p>
                                </div>
                            </div>
                        </div>
                    </div>
                </section>

                // ── Comparison ──────────────────────────────────────────────
                <section class="pub-section">
                    <div class="section-inner">
                        <div class="section-tag">"Why cloudMailer"</div>
                        <h2 class="section-title">"Dedicated ESPs vs cloudMailer"</h2>
                        <p class="section-sub">
                            "Most providers own your sending pipeline. cloudMailer is a pure relay — "
                            "your credentials, your domain, your data, your rules. "
                            <strong>"It's completely free to use."</strong>
                            " Sign up, wire your SMTP transport, and start sending."
                        </p>
                        <div class="free-banner">
                            <span class="free-icon">"🎉"</span>
                            <div>
                                <strong>"Free to use — no credit card, no per-email fees."</strong>
                                <span>" Log in · Add a transport · Send. That's it."</span>
                            </div>
                        </div>
                        <div class="compare-wrap">
                            <table class="compare-table">
                                <thead>
                                    <tr>
                                        <th class="compare-feature"></th>
                                        <th class="compare-col">"SendGrid / Mailgun / SES"</th>
                                        <th class="compare-col compare-ours">"☁ cloudMailer"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    <tr>
                                        <td class="compare-feature">"Pricing"</td>
                                        <td class="cmp-no">"✗  Per-email fees"</td>
                                        <td class="cmp-yes compare-ours">"✓  Free — always"</td>
                                    </tr>
                                    <tr>
                                        <td class="compare-feature">"Sending credentials"</td>
                                        <td class="cmp-no">"✗  Vendor-owned"</td>
                                        <td class="cmp-yes compare-ours">"✓  Yours, AES-256-GCM encrypted"</td>
                                    </tr>
                                    <tr>
                                        <td class="compare-feature">"Domain ownership"</td>
                                        <td class="cmp-no">"✗  Shared / vendor"</td>
                                        <td class="cmp-yes compare-ours">"✓  Fully yours"</td>
                                    </tr>
                                    <tr>
                                        <td class="compare-feature">"Delivery retries"</td>
                                        <td class="cmp-yes">"✓  Yes"</td>
                                        <td class="cmp-yes compare-ours">"✓  Yes (configurable)"</td>
                                    </tr>
                                    <tr>
                                        <td class="compare-feature">"Vendor lock-in"</td>
                                        <td class="cmp-no">"✗  High"</td>
                                        <td class="cmp-yes compare-ours">"✓  None"</td>
                                    </tr>
                                    <tr>
                                        <td class="compare-feature">"Template engine"</td>
                                        <td class="cmp-no">"✗  Proprietary DSL"</td>
                                        <td class="cmp-yes compare-ours">"✓  Open Handlebars — optional"</td>
                                    </tr>
                                    <tr>
                                        <td class="compare-feature">"Raw email (no template)"</td>
                                        <td class="cmp-no">"✗  Not standard"</td>
                                        <td class="cmp-yes compare-ours">"✓  Pass body_html directly"</td>
                                    </tr>
                                </tbody>
                            </table>
                        </div>
                    </div>
                </section>

                // ── Final CTA ───────────────────────────────────────────────
                <section class="cta-section">
                    <div class="section-inner">
                        <h2>"Start sending in under two minutes."</h2>
                        <p>
                            "No credit card. No per-email fees. "
                            "Log in → add your SMTP transport → call the API. Done."
                        </p>
                        <div class="hero-cta">
                            <a href="/signup" class="btn-hero">"Create free account"</a>
                            <a href="/roadmap" class="btn-hero-ghost">"See what's coming →"</a>
                        </div>
                    </div>
                </section>

            </PublicLayout>
        }
    });
    Html(html).into_response()
}
