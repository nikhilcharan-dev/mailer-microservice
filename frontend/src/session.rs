use axum::http::HeaderMap;
use axum::response::Redirect;
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};

pub const SESSION_COOKIE: &str = "central_mailer_session";

pub fn token_from_jar(jar: &CookieJar) -> Option<String> {
    jar.get(SESSION_COOKIE).map(|c| c.value().to_string())
}

#[allow(dead_code)] // helper for code paths not using the CookieJar extractor
pub fn token_from_headers(headers: &HeaderMap) -> Option<String> {
    let cookie_hdr = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;
    for part in cookie_hdr.split(';') {
        let p = part.trim();
        if let Some(v) = p.strip_prefix(&format!("{SESSION_COOKIE}=")) {
            return Some(v.to_string());
        }
    }
    None
}

pub fn set_token(jar: CookieJar, token: String) -> CookieJar {
    let cookie = Cookie::build((SESSION_COOKIE, token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::hours(24))
        .build();
    jar.add(cookie)
}

pub fn clear_token(jar: CookieJar) -> CookieJar {
    let cookie = Cookie::build((SESSION_COOKIE, ""))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::seconds(0))
        .build();
    jar.add(cookie)
}

pub fn redirect_to_login() -> Redirect {
    Redirect::to("/login")
}

