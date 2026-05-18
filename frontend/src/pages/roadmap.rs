use axum::response::{Html, IntoResponse, Response};
use leptos::prelude::*;

use crate::ui::{render_page, PublicLayout};

pub async fn roadmap_page() -> Response {
    let html = render_page(|| {
        view! {
            <PublicLayout title="Roadmap — Future Features">
                <section class="pub-section roadmap-hero">
                    <div class="section-inner" style="text-align:center;">
                        <div class="section-tag">"Roadmap"</div>
                        <h1 class="section-title">"What's coming to cloudMailer"</h1>
                        <p class="section-sub">
                            "cloudMailer ships production-grade primitives today. "
                            "Here's everything we're building next — in priority order."
                        </p>
                    </div>
                </section>

                // ── Coming Soon ─────────────────────────────────────────────
                <section class="pub-section">
                    <div class="section-inner">
                        <div class="rm-group-header">
                            <span class="rm-badge soon">"Coming soon"</span>
                            <h2>"Next up"</h2>
                        </div>
                        <div class="roadmap-grid">

                            <div class="rm-card">
                                <div class="rm-icon">"🔑"</div>
                                <h3>"OAuth2 Gmail &amp; Outlook"</h3>
                                <p>
                                    "Replace app-password workarounds with real OAuth2 flows. "
                                    "Click-to-authorize your Gmail or Microsoft 365 account — "
                                    "no SMTP passwords stored at all."
                                </p>
                                <div class="rm-tags">
                                    <span class="rm-tag">"Security"</span>
                                    <span class="rm-tag">"UX"</span>
                                </div>
                            </div>

                            <div class="rm-card">
                                <div class="rm-icon">"🔔"</div>
                                <h3>"Delivery Webhooks"</h3>
                                <p>
                                    "Fire a configurable HTTP webhook on every send outcome — "
                                    "success, failure, or timeout. Integrate with your own alerting, "
                                    "CRM, or analytics pipeline."
                                </p>
                                <div class="rm-tags">
                                    <span class="rm-tag">"Integrations"</span>
                                </div>
                            </div>

                            <div class="rm-card">
                                <div class="rm-icon">"⚠️"</div>
                                <h3>"Bounce &amp; Complaint Handling"</h3>
                                <p>
                                    "Automatic suppression of hard-bounced addresses and complaint-flagged "
                                    "recipients. Protects your sending reputation and keeps logs clean."
                                </p>
                                <div class="rm-tags">
                                    <span class="rm-tag">"Deliverability"</span>
                                </div>
                            </div>

                            <div class="rm-card">
                                <div class="rm-icon">"📊"</div>
                                <h3>"Email Analytics Dashboard"</h3>
                                <p>
                                    "Open rate, click-through rate, and bounce rate per template and transport. "
                                    "Time-series charts built into the existing dashboard — "
                                    "no third-party trackers."
                                </p>
                                <div class="rm-tags">
                                    <span class="rm-tag">"Analytics"</span>
                                </div>
                            </div>

                        </div>
                    </div>
                </section>

                // ── Planned ─────────────────────────────────────────────────
                <section class="pub-section alt">
                    <div class="section-inner">
                        <div class="rm-group-header">
                            <span class="rm-badge planned">"Planned"</span>
                            <h2>"On the roadmap"</h2>
                        </div>
                        <div class="roadmap-grid">

                            <div class="rm-card">
                                <div class="rm-icon">"👥"</div>
                                <h3>"Team Workspaces &amp; RBAC"</h3>
                                <p>
                                    "Invite teammates to a shared workspace with role-based access control. "
                                    "Admins manage transports; developers call the send API — "
                                    "no credential sharing."
                                </p>
                                <div class="rm-tags">
                                    <span class="rm-tag">"Teams"</span>
                                    <span class="rm-tag">"Security"</span>
                                </div>
                            </div>

                            <div class="rm-card">
                                <div class="rm-icon">"🗝"</div>
                                <h3>"Multiple API Key Tiers"</h3>
                                <p>
                                    "Create scoped API keys per project or service — each with its own "
                                    "rate limit, expiry, and transport whitelist. "
                                    "Rotate one key without affecting others."
                                </p>
                                <div class="rm-tags">
                                    <span class="rm-tag">"Security"</span>
                                    <span class="rm-tag">"DevEx"</span>
                                </div>
                            </div>

                            <div class="rm-card">
                                <div class="rm-icon">"📥"</div>
                                <h3>"Inbound Email Parsing"</h3>
                                <p>
                                    "Receive inbound mail on a configured address and forward parsed "
                                    "metadata (from, subject, body) to a webhook endpoint. "
                                    "Build reply workflows or support inboxes without a mail server."
                                </p>
                                <div class="rm-tags">
                                    <span class="rm-tag">"Inbound"</span>
                                    <span class="rm-tag">"Integrations"</span>
                                </div>
                            </div>

                            <div class="rm-card">
                                <div class="rm-icon">"⏰"</div>
                                <h3>"Email Scheduling &amp; Batch Sends"</h3>
                                <p>
                                    "Schedule an email for a future time or send to a list of recipients "
                                    "in a single API call with per-recipient variable interpolation. "
                                    "Redis-backed job queue with retries."
                                </p>
                                <div class="rm-tags">
                                    <span class="rm-tag">"Scheduling"</span>
                                    <span class="rm-tag">"Bulk"</span>
                                </div>
                            </div>

                        </div>
                    </div>
                </section>

                // ── Exploring ───────────────────────────────────────────────
                <section class="pub-section">
                    <div class="section-inner">
                        <div class="rm-group-header">
                            <span class="rm-badge exploring">"Exploring"</span>
                            <h2>"Under consideration"</h2>
                        </div>
                        <div class="roadmap-grid roadmap-grid-wide">

                            <div class="rm-card rm-card-sm">
                                <div class="rm-icon">"✉️"</div>
                                <h3>"DKIM / SPF Setup Wizard"</h3>
                                <p>"Guided DNS record generation for your sending domain."</p>
                            </div>

                            <div class="rm-card rm-card-sm">
                                <div class="rm-icon">"🌍"</div>
                                <h3>"Multi-region Deployment"</h3>
                                <p>"Deploy shards closer to your users; route sends to the nearest region."</p>
                            </div>

                            <div class="rm-card rm-card-sm">
                                <div class="rm-icon">"💬"</div>
                                <h3>"Slack / Discord Alerts"</h3>
                                <p>"Push delivery failures or quota warnings straight to your team channel."</p>
                            </div>

                            <div class="rm-card rm-card-sm">
                                <div class="rm-icon">"📝"</div>
                                <h3>"Template Versioning"</h3>
                                <p>"Keep a full history of template edits; roll back to any previous version."</p>
                            </div>

                            <div class="rm-card rm-card-sm">
                                <div class="rm-icon">"📋"</div>
                                <h3>"CSV Bulk Send"</h3>
                                <p>"Upload a CSV and map columns to template variables for one-click campaigns."</p>
                            </div>

                            <div class="rm-card rm-card-sm">
                                <div class="rm-icon">"🚫"</div>
                                <h3>"Custom Unsubscribe Links"</h3>
                                <p>"Auto-inject one-click unsubscribe headers and honour suppression lists."</p>
                            </div>

                        </div>
                    </div>
                </section>

                // ── CTA ────────────────────────────────────────────────────
                <section class="cta-section">
                    <div class="section-inner">
                        <h2>"Have a feature idea?"</h2>
                        <p>"cloudMailer is open-source. Open an issue or submit a PR — we ship fast."</p>
                        <div class="hero-cta">
                            <a href="/signup" class="btn-hero">"Get started free"</a>
                            <a href="/docs" class="btn-hero-ghost">"Read the docs →"</a>
                        </div>
                    </div>
                </section>

            </PublicLayout>
        }
    });
    Html(html).into_response()
}
