use anyhow::Result;
use reqwest::{Client, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::sync::OnceLock;

fn client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .expect("reqwest client")
    })
}

fn base_url() -> String {
    std::env::var("BACKEND_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".into())
}

pub struct ApiError {
    #[allow(dead_code)] // useful for callers that want to branch on HTTP status
    pub status: StatusCode,
    pub code: String,
    pub message: String,
}

impl ApiError {
    pub fn from_value(status: StatusCode, v: Value) -> Self {
        let code = v
            .get("code")
            .and_then(|x| x.as_str())
            .unwrap_or("error")
            .to_string();
        let message = v
            .get("error")
            .and_then(|x| x.as_str())
            .unwrap_or("an error occurred")
            .to_string();
        ApiError { status, code, message }
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

async fn request_inner(
    method: reqwest::Method,
    path: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> Result<(StatusCode, Value)> {
    let url = format!("{}{}", base_url(), path);
    let mut req = client().request(method, url);
    if let Some(t) = token {
        req = req.header("Authorization", format!("Bearer {t}"));
    }
    if let Some(b) = body {
        req = req.json(&b);
    }
    let res = req.send().await?;
    let status = res.status();
    let text = res.text().await?;
    let value: Value = if text.is_empty() {
        Value::Null
    } else {
        serde_json::from_str(&text).unwrap_or(Value::String(text))
    };
    Ok((status, value))
}

#[allow(dead_code)] // typed-response helper, kept as API surface
pub async fn request_json<T: DeserializeOwned>(
    method: reqwest::Method,
    path: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> ApiResult<T> {
    let (status, value) = request_inner(method, path, token, body)
        .await
        .map_err(|e| ApiError {
            status: StatusCode::BAD_GATEWAY,
            code: "backend_unreachable".into(),
            message: e.to_string(),
        })?;
    if !status.is_success() {
        return Err(ApiError::from_value(status, value));
    }
    serde_json::from_value(value).map_err(|e| ApiError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        code: "decode_error".into(),
        message: e.to_string(),
    })
}

pub async fn request_value(
    method: reqwest::Method,
    path: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> ApiResult<Value> {
    let (status, value) = request_inner(method, path, token, body)
        .await
        .map_err(|e| ApiError {
            status: StatusCode::BAD_GATEWAY,
            code: "backend_unreachable".into(),
            message: e.to_string(),
        })?;
    if !status.is_success() {
        return Err(ApiError::from_value(status, value));
    }
    Ok(value)
}
