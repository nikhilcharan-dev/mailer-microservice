pub mod api_key;
pub mod rate_limit;
pub mod template;
pub mod transport;

use anyhow::{Context, Result};
use redis::aio::ConnectionManager;

pub async fn connect() -> Result<ConnectionManager> {
    let host = std::env::var("REDIS_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port = std::env::var("REDIS_PORT").unwrap_or_else(|_| "6379".into());
    let pass = std::env::var("REDIS_PASSWORD").unwrap_or_default();
    let url = if pass.is_empty() {
        format!("redis://{host}:{port}")
    } else {
        format!("redis://:{pass}@{host}:{port}")
    };
    let client = redis::Client::open(url).context("invalid REDIS_URL")?;
    let manager = ConnectionManager::new(client)
        .await
        .context("redis connect failed")?;
    Ok(manager)
}

pub async fn ping(conn: &ConnectionManager) -> Result<()> {
    let mut c = conn.clone();
    let _: String = redis::cmd("PING").query_async(&mut c).await?;
    Ok(())
}

/// Add 0-N seconds of random jitter to a base TTL to spread expirations.
pub fn jitter(base_secs: u64, max_jitter_secs: u64) -> u64 {
    use rand::Rng;
    base_secs + rand::thread_rng().gen_range(0..=max_jitter_secs)
}
