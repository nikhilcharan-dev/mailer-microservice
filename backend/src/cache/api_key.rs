use anyhow::Result;
use redis::{aio::ConnectionManager, AsyncCommands};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedApiKey {
    pub user_id: String,
    pub username: String,
    pub daily_limit: u32,
}

const TTL: u64 = 15 * 60;

fn key(api_key: &str) -> String {
    format!("apikey:{api_key}")
}

pub async fn get(conn: &ConnectionManager, api_key: &str) -> Result<Option<CachedApiKey>> {
    let mut c = conn.clone();
    let val: Option<String> = c.get(key(api_key)).await?;
    Ok(val.and_then(|s| serde_json::from_str(&s).ok()))
}

pub async fn set(conn: &ConnectionManager, api_key: &str, v: &CachedApiKey) -> Result<()> {
    let mut c = conn.clone();
    let json = serde_json::to_string(v)?;
    c.set_ex::<_, _, ()>(key(api_key), json, TTL).await?;
    Ok(())
}

pub async fn invalidate(conn: &ConnectionManager, api_key: &str) -> Result<()> {
    let mut c = conn.clone();
    c.del::<_, ()>(key(api_key)).await?;
    Ok(())
}
