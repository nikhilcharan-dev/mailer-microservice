use anyhow::Result;
use redis::{aio::ConnectionManager, AsyncCommands};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedTransport {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub tls_mode: String,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub from_name: String,
    pub from_email: String,
}

const TTL: u64 = 5 * 60;
const NEG_TTL: u64 = 30;

fn key(user_id: &str, name: &str) -> String {
    format!("transport:{user_id}:{name}")
}

fn neg_key(user_id: &str, name: &str) -> String {
    format!("negcache:transport:{user_id}:{name}")
}

pub async fn get(
    conn: &ConnectionManager,
    user_id: &str,
    name: &str,
) -> Result<Option<CachedTransport>> {
    let mut c = conn.clone();
    let val: Option<String> = c.get(key(user_id, name)).await?;
    Ok(val.and_then(|s| serde_json::from_str(&s).ok()))
}

pub async fn set(
    conn: &ConnectionManager,
    user_id: &str,
    name: &str,
    v: &CachedTransport,
) -> Result<()> {
    let mut c = conn.clone();
    let json = serde_json::to_string(v)?;
    c.set_ex::<_, _, ()>(key(user_id, name), json, TTL).await?;
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
