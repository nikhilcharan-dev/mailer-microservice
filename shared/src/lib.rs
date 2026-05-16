use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── Auth ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct SignupRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignupResponse {
    pub status: String,
    pub api_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SigninRequest {
    pub identifier: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SigninResponse {
    pub status: String,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiKeyResponse {
    pub status: String,
    pub api_key: String,
}

// ── Transports ───────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTransportRequest {
    pub name: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub tls_mode: String,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub from_name: String,
    pub from_email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTransportRequest {
    pub smtp_host: Option<String>,
    pub smtp_port: Option<u16>,
    pub tls_mode: Option<String>,
    pub smtp_user: Option<String>,
    pub smtp_pass: Option<String>,
    pub from_name: Option<String>,
    pub from_email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransportView {
    pub name: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub tls_mode: String,
    pub smtp_user: String,
    pub from_name: String,
    pub from_email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── Templates ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTemplateRequest {
    pub name: String,
    pub subject: String,
    pub html: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTemplateRequest {
    pub subject: Option<String>,
    pub html: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateView {
    pub name: String,
    pub subject: String,
    pub variables: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateDetail {
    pub name: String,
    pub subject: String,
    pub html: String,
    pub variables: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── Send ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct SendSuccess {
    pub status: String,
    pub message_id: String,
}

// ── Logs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct LogView {
    pub id: String,
    pub transport_name: String,
    pub template_name: String,
    pub to: String,
    pub status: String,
    pub error: Option<String>,
    pub duration_ms: u32,
    pub sent_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogListResponse {
    pub logs: Vec<LogView>,
    pub total: u64,
    pub page: u64,
    pub pages: u64,
}

// ── Account ──────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountView {
    pub username: String,
    pub email: String,
    pub daily_limit: u32,
    pub sent_today: u64,
    pub transport_count: u64,
    pub template_count: u64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteAccountRequest {
    pub password: String,
}

// ── Generic responses ────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusResponse {
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub status: String,
    pub code: String,
    pub error: String,
}
