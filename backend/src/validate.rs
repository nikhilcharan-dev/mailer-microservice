use once_cell::sync::Lazy;
use regex::Regex;

static EMAIL_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap());

static USERNAME_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-z0-9](?:[a-z0-9_-]{1,30}[a-z0-9])?$").unwrap());

static NAME_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z0-9_-]{1,64}$").unwrap());

const RESERVED_USERNAMES: &[&str] = &[
    "admin", "api", "system", "root", "support", "null", "undefined", "auth", "v1",
    "send", "transports", "templates", "logs", "account", "health", "ready",
    "metrics", "docs", "www",
];

pub fn is_email(s: &str) -> bool {
    s.len() <= 254 && EMAIL_RE.is_match(s)
}

pub fn is_valid_username(s: &str) -> bool {
    if RESERVED_USERNAMES.contains(&s) {
        return false;
    }
    USERNAME_RE.is_match(s)
}

pub fn is_valid_resource_name(s: &str) -> bool {
    NAME_RE.is_match(s)
}

pub fn is_strong_password(s: &str) -> bool {
    s.len() >= 12
}

pub fn is_valid_tls_mode(s: &str) -> bool {
    matches!(s, "starttls" | "tls" | "none")
}
