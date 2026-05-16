use anyhow::Result;
use futures::stream::TryStreamExt;
use mongodb::bson::{doc, oid::ObjectId, DateTime, Document};
use mongodb::options::FindOptions;
use mongodb::Database;

use crate::models::log::LogDoc;

pub fn col(db: &Database) -> mongodb::Collection<LogDoc> {
    db.collection("delivery_logs")
}

pub async fn insert(db: &Database, doc: &LogDoc) -> Result<()> {
    col(db).insert_one(doc, None).await?;
    Ok(())
}

pub struct LogFilter<'a> {
    pub user_id: &'a ObjectId,
    pub transport: Option<&'a str>,
    pub template: Option<&'a str>,
    pub status: Option<&'a str>,
}

fn build_filter(f: &LogFilter) -> Document {
    let mut q = doc! { "user_id": f.user_id };
    if let Some(t) = f.transport {
        q.insert("transport_name", t);
    }
    if let Some(t) = f.template {
        q.insert("template_name", t);
    }
    if let Some(s) = f.status {
        q.insert("status", s);
    }
    q
}

pub async fn count(db: &Database, filter: &LogFilter<'_>) -> Result<u64> {
    Ok(col(db).count_documents(build_filter(filter), None).await?)
}

pub async fn list(
    db: &Database,
    filter: &LogFilter<'_>,
    page: u64,
    limit: i64,
) -> Result<Vec<LogDoc>> {
    let opts = FindOptions::builder()
        .sort(doc! { "sent_at": -1 })
        .skip(Some((page.saturating_sub(1)) * limit as u64))
        .limit(limit)
        .build();
    let cursor = col(db).find(build_filter(filter), opts).await?;
    Ok(cursor.try_collect().await?)
}

/// Counts user's sends since the given DateTime — used to backfill sent_today
/// when the Redis counter is unavailable.
pub async fn count_since(db: &Database, user_id: &ObjectId, since: DateTime) -> Result<u64> {
    Ok(col(db)
        .count_documents(
            doc! { "user_id": user_id, "sent_at": { "$gte": since }, "status": "success" },
            None,
        )
        .await?)
}
