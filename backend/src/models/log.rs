use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogDoc {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    pub transport_name: String,
    pub template_name: String,
    pub to: String,
    pub status: String,
    pub error: Option<String>,
    pub duration_ms: u32,
    pub sent_at: DateTime,
}
