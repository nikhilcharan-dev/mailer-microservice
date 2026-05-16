use std::sync::Arc;

use crate::auth::jwt::JwtConfig;
use crate::crypto::aes::EncryptionKeyring;
use crate::db::Db;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Db>,
    pub redis: redis::aio::ConnectionManager,
    pub keyring: Arc<EncryptionKeyring>,
    pub jwt: Arc<JwtConfig>,
    pub limits: Arc<Limits>,
}

pub struct Limits {
    pub default_daily: u32,
    pub max_transports: u64,
    pub max_templates: u64,
    pub max_html_bytes: usize,
    pub max_subject_chars: usize,
    #[allow(dead_code)] // reserved for future HTTPS-enforcement middleware
    pub require_https: bool,
}

impl Limits {
    pub fn from_env() -> Self {
        let parse_u = |k: &str, d: u64| -> u64 {
            std::env::var(k).ok().and_then(|s| s.parse().ok()).unwrap_or(d)
        };
        let parse_us = |k: &str, d: usize| -> usize {
            std::env::var(k).ok().and_then(|s| s.parse().ok()).unwrap_or(d)
        };
        Self {
            default_daily: parse_u("DEFAULT_DAILY_LIMIT", 500) as u32,
            max_transports: parse_u("MAX_TRANSPORTS_PER_USER", 10),
            max_templates: parse_u("MAX_TEMPLATES_PER_USER", 50),
            max_html_bytes: parse_us("MAX_HTML_BYTES", 1_048_576), // 1 MiB
            max_subject_chars: parse_us("MAX_SUBJECT_CHARS", 998),
            require_https: std::env::var("REQUIRE_HTTPS")
                .map(|v| v == "true")
                .unwrap_or(false),
        }
    }
}
