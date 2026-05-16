use anyhow::Result;
use redis::{aio::ConnectionManager, AsyncCommands};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedTemplate {
    pub subject: String,
    pub html: String,
    pub variables: Vec<String>,
}

const BASE_TTL: u64 = 60 * 60;
const JITTER: u64 = 10 * 60;
const NEG_TTL: u64 = 30;

fn key(user_id: &str, name: &str) -> String {
    format!("template:{user_id}:{name}")
}

fn neg_key(user_id: &str, name: &str) -> String {
    format!("negcache:template:{user_id}:{name}")
}

pub async fn get(
    conn: &ConnectionManager,
    user_id: &str,
    name: &str,
) -> Result<Option<CachedTemplate>> {
    let mut c = conn.clone();
    let val: Option<String> = c.get(key(user_id, name)).await?;
    Ok(val.and_then(|s| serde_json::from_str(&s).ok()))
}

pub async fn set(
    conn: &ConnectionManager,
    user_id: &str,
    name: &str,
    v: &CachedTemplate,
) -> Result<()> {
    let mut c = conn.clone();
    let json = serde_json::to_string(v)?;
    let ttl = super::jitter(BASE_TTL, JITTER);
    c.set_ex::<_, _, ()>(key(user_id, name), json, ttl).await?;
    Ok(())
}

pub async fn invalidate(conn: &ConnectionManager, user_id: &str, name: &str) -> Result<()> {
    let mut c = conn.clone();
    c.del::<_, ()>(key(user_id, name)).await?;
    c.del::<_, ()>(neg_key(user_id, name)).await?;
    Ok(())
}

pub async fn negcache_set(conn: &ConnectionManager, user_id: &str, name: &str) -> Result<()> {
    let mut c = conn.clone();
    c.set_ex::<_, _, ()>(neg_key(user_id, name), "1", NEG_TTL)
        .await?;
    Ok(())
}

pub async fn negcache_hit(conn: &ConnectionManager, user_id: &str, name: &str) -> Result<bool> {
    let mut c = conn.clone();
    let val: Option<String> = c.get(neg_key(user_id, name)).await?;
    Ok(val.is_some())
}
