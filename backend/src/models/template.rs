use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateDoc {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    pub name: String,
    pub subject: String,
    pub html: String,
    pub variables: Vec<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}
