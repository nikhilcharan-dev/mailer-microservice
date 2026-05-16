use anyhow::Result;
use chrono::{Duration, NaiveTime, Utc};
use once_cell::sync::Lazy;
use redis::{aio::ConnectionManager, AsyncCommands, Script};

/// Atomic INCR + EXPIREAT-on-first.
/// KEYS[1] = the counter key, ARGV[1] = unix timestamp of expiry.
/// Returns the new count after increment.
static RESERVE_SCRIPT: Lazy<Script> = Lazy::new(|| {
    Script::new(
        r#"
        local n = redis.call("INCR", KEYS[1])
        if n == 1 then redis.call("EXPIREAT", KEYS[1], ARGV[1]) end
        return n
        "#,
    )
});

fn daily_key(user_id: &str) -> String {
    format!("ratelimit:daily:{user_id}")
}

fn next_midnight_unix() -> i64 {
    let now = Utc::now();
    let today_midnight = now.date_naive().and_time(NaiveTime::MIN);
    let next = today_midnight + Duration::days(1);
    next.and_utc().timestamp()
}

/// Atomically reserve a slot from the daily quota.
/// Returns the new count after INCR. Caller must DECR if count > limit.
pub async fn reserve_daily(conn: &ConnectionManager, user_id: &str) -> Result<i64> {
    let mut c = conn.clone();
    let key = daily_key(user_id);
    let ts = next_midnight_unix();
    let n: i64 = RESERVE_SCRIPT
        .key(key)
        .arg(ts)
        .invoke_async(&mut c)
        .await?;
    Ok(n)
}

pub async fn release_daily(conn: &ConnectionManager, user_id: &str) {
    let mut c = conn.clone();
    let _: Result<i64, _> = c.decr(daily_key(user_id), 1).await;
}

pub async fn get_daily(conn: &ConnectionManager, user_id: &str) -> Result<i64> {
    let mut c = conn.clone();
    let val: Option<i64> = c.get(daily_key(user_id)).await?;
    Ok(val.unwrap_or(0))
}

// ── Signup throttle ──────────────────────────────────────────────────────────

const SIGNUP_LIMIT: i64 = 5;
const SIGNUP_WINDOW_SECS: u64 = 3600;

static SIGNUP_SCRIPT: Lazy<Script> = Lazy::new(|| {
    Script::new(
        r#"
        local n = redis.call("INCR", KEYS[1])
        if n == 1 then redis.call("EXPIRE", KEYS[1], ARGV[1]) end
        return n
        "#,
    )
});

pub async fn check_signup(conn: &ConnectionManager, ip: &str) -> Result<bool> {
    let mut c = conn.clone();
    let key = format!("signup:ip:{ip}");
    let n: i64 = SIGNUP_SCRIPT
        .key(key)
        .arg(SIGNUP_WINDOW_SECS)
        .invoke_async(&mut c)
        .await?;
    Ok(n <= SIGNUP_LIMIT)
}

// ── Signin failure throttle ──────────────────────────────────────────────────

const SIGNIN_LIMIT: i64 = 10;
const SIGNIN_WINDOW_SECS: u64 = 15 * 60;

pub async fn check_signin(conn: &ConnectionManager, ip: &str) -> Result<bool> {
    let mut c = conn.clone();
    let key = format!("signin:fail:{ip}");
    let val: Option<i64> = c.get(&key).await?;
    Ok(val.unwrap_or(0) < SIGNIN_LIMIT)
}

pub async fn record_signin_failure(conn: &ConnectionManager, ip: &str) -> Result<()> {
    let mut c = conn.clone();
    let key = format!("signin:fail:{ip}");
    let n: i64 = SIGNUP_SCRIPT
        .key(key)
        .arg(SIGNIN_WINDOW_SECS)
        .invoke_async(&mut c)
        .await?;
    let _ = n;
    Ok(())
}

// ── Verify-transport throttle ────────────────────────────────────────────────

const VERIFY_LIMIT: i64 = 5;
const VERIFY_WINDOW_SECS: u64 = 60;

pub async fn check_verify(conn: &ConnectionManager, user_id: &str) -> Result<bool> {
    let mut c = conn.clone();
    let key = format!("verify:{user_id}");
    let n: i64 = SIGNUP_SCRIPT
        .key(key)
        .arg(VERIFY_WINDOW_SECS)
        .invoke_async(&mut c)
        .await?;
    Ok(n <= VERIFY_LIMIT)
}
