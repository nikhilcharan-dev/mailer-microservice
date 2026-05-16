use anyhow::Result;
use mongodb::bson::{doc, oid::ObjectId, Document};
use mongodb::Database;
use futures::stream::TryStreamExt;

use crate::models::template::TemplateDoc;

pub fn col(db: &Database) -> mongodb::Collection<TemplateDoc> {
    db.collection("templates")
}

pub async fn insert(db: &Database, doc: &TemplateDoc) -> Result<()> {
    col(db).insert_one(doc, None).await?;
    Ok(())
}

pub async fn find(db: &Database, user_id: &ObjectId, name: &str) -> Result<Option<TemplateDoc>> {
    Ok(col(db)
        .find_one(doc! { "user_id": user_id, "name": name }, None)
        .await?)
}

pub async fn list(db: &Database, user_id: &ObjectId) -> Result<Vec<TemplateDoc>> {
    let cursor = col(db).find(doc! { "user_id": user_id }, None).await?;
    Ok(cursor.try_collect().await?)
}

pub async fn count(db: &Database, user_id: &ObjectId) -> Result<u64> {
    Ok(col(db)
        .count_documents(doc! { "user_id": user_id }, None)
        .await?)
}

pub async fn update(db: &Database, user_id: &ObjectId, name: &str, set: Document) -> Result<bool> {
    let res = col(db)
        .update_one(
            doc! { "user_id": user_id, "name": name },
            doc! { "$set": set },
            None,
        )
        .await?;
    Ok(res.matched_count > 0)
}

pub async fn delete(db: &Database, user_id: &ObjectId, name: &str) -> Result<bool> {
    let res = col(db)
        .delete_one(doc! { "user_id": user_id, "name": name }, None)
        .await?;
    Ok(res.deleted_count > 0)
}

pub async fn delete_all_for_user(db: &Database, user_id: &ObjectId) -> Result<u64> {
    let res = col(db).delete_many(doc! { "user_id": user_id }, None).await?;
    Ok(res.deleted_count)
}
