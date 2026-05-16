use mongodb::bson::{oid::ObjectId, Binary, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportDoc {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    pub name: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub tls_mode: String,
    pub smtp_user: String,
    pub smtp_pass_enc: Binary,
    pub smtp_pass_nonce: Binary,
    pub key_version: u32,
    pub from_name: String,
    pub from_email: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}
