use anyhow::{anyhow, Context, Result};
use lettre::{
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use std::time::Duration;
use tokio::time::timeout;

use crate::cache::transport::CachedTransport;

const SMTP_TIMEOUT_SECS: u64 = 10;

pub enum SendOutcome {
    Ok(String), // message_id (best-effort)
    Failed(String),
    TimedOut,
}

pub fn build_transport(cfg: &CachedTransport) -> Result<AsyncSmtpTransport<Tokio1Executor>> {
    let creds = Credentials::new(cfg.smtp_user.clone(), cfg.smtp_pass.clone());
    let builder = match cfg.tls_mode.as_str() {
        "tls" => AsyncSmtpTransport::<Tokio1Executor>::relay(&cfg.smtp_host)?,
        "starttls" => AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&cfg.smtp_host)?,
        "none" => AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&cfg.smtp_host),
        other => return Err(anyhow!("invalid tls_mode: {other}")),
    };
    Ok(builder
        .port(cfg.smtp_port)
        .credentials(creds)
        .timeout(Some(Duration::from_secs(SMTP_TIMEOUT_SECS)))
        .build())
}

pub async fn send(
    transport: &AsyncSmtpTransport<Tokio1Executor>,
    cfg: &CachedTransport,
    to: &str,
    subject: &str,
    html: String,
) -> SendOutcome {
    let from_addr = match format!("{} <{}>", cfg.from_name, cfg.from_email).parse() {
        Ok(a) => a,
        Err(e) => return SendOutcome::Failed(format!("bad from address: {e}")),
    };
    let to_addr = match to.parse() {
        Ok(a) => a,
        Err(e) => return SendOutcome::Failed(format!("bad to address: {e}")),
    };

    let message = match Message::builder()
        .from(from_addr)
        .to(to_addr)
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(html)
    {
        Ok(m) => m,
        Err(e) => return SendOutcome::Failed(format!("message build failed: {e}")),
    };

    match timeout(Duration::from_secs(SMTP_TIMEOUT_SECS + 2), transport.send(message)).await {
        Ok(Ok(resp)) => {
            // lettre's SmtpResponse has a Display impl; the message-id isn't directly
            // exposed in 0.11, so we use the response code/message as the identifier.
            let mid = format!("{resp:?}").chars().take(120).collect::<String>();
            SendOutcome::Ok(mid)
        }
        Ok(Err(e)) => {
            let s = e.to_string();
            SendOutcome::Failed(truncate(&s, 500))
        }
        Err(_) => SendOutcome::TimedOut,
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}

/// Verify SMTP credentials by connecting (without sending a message).
pub async fn verify(cfg: &CachedTransport) -> Result<()> {
    let t = build_transport(cfg)?;
    timeout(Duration::from_secs(SMTP_TIMEOUT_SECS), t.test_connection())
        .await
        .context("SMTP verify timed out")?
        .context("SMTP test connection failed")?;
    Ok(())
}
