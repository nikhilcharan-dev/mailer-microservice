use anyhow::Result;
use mongodb::bson::{doc, oid::ObjectId};
use mongodb::Database;

use crate::models::user::UserDoc;

pub fn col(db: &Database) -> mongodb::Collection<UserDoc> {
    db.collection("users")
}

pub async fn insert(db: &Database, doc: &UserDoc) -> Result<ObjectId> {
    let res = col(db).insert_one(doc, None).await?;
    Ok(res
        .inserted_id
        .as_object_id()
        .ok_or_else(|| anyhow::anyhow!("inserted_id not ObjectId"))?)
}

pub async fn find_by_email(db: &Database, email: &str) -> Result<Option<UserDoc>> {
    Ok(col(db).find_one(doc! { "email": email }, None).await?)
}

pub async fn find_by_username(db: &Database, username: &str) -> Result<Option<UserDoc>> {
    Ok(col(db).find_one(doc! { "username": username }, None).await?)
}

pub async fn find_by_identifier(db: &Database, identifier: &str) -> Result<Option<UserDoc>> {
    Ok(col(db)
        .find_one(
            doc! { "$or": [ { "email": identifier }, { "username": identifier } ] },
            None,
        )
        .await?)
}

pub async fn find_by_api_key(db: &Database, api_key: &str) -> Result<Option<UserDoc>> {
    Ok(col(db).find_one(doc! { "api_key": api_key }, None).await?)
}

pub async fn find_by_id(db: &Database, id: &ObjectId) -> Result<Option<UserDoc>> {
    Ok(col(db).find_one(doc! { "_id": id }, None).await?)
}

pub async fn rotate_api_key(db: &Database, id: &ObjectId, new_key: &str) -> Result<()> {
    col(db)
        .update_one(
            doc! { "_id": id },
            doc! { "$set": { "api_key": new_key } },
            None,
        )
        .await?;
    Ok(())
}

pub async fn delete(db: &Database, id: &ObjectId) -> Result<()> {
    col(db).delete_one(doc! { "_id": id }, None).await?;
    Ok(())
}
