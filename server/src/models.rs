use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct RegisterDeviceRequest {
    #[serde(rename = "deviceId")]
    pub device_id: String,
    #[serde(default = "default_channel")]
    pub channel: String,
    #[serde(default)]
    pub auth: ChannelAuth,
}

#[derive(Debug, Serialize)]
pub struct RegisterDeviceResponse {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct PushRequest {
    #[serde(default = "default_channel")]
    pub channel: String,
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub extras: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct PushResponse {
    pub id: String,
    pub accepted: bool,
    pub online_deliveries: usize,
}

#[derive(Debug, Deserialize)]
pub struct DeleteMessagesRequest {
    pub ids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct DeleteMessagesResponse {
    pub deleted: u64,
}

#[derive(Debug, Serialize)]
pub struct DeleteChannelResponse {
    pub deleted_channel: bool,
    pub deleted_messages: u64,
}

#[derive(Debug, Deserialize)]
pub struct SyncQuery {
    #[serde(rename = "deviceId")]
    pub device_id: String,
    #[serde(default = "default_channel")]
    pub channel: String,
    #[serde(default)]
    pub ts: String,
    #[serde(default)]
    pub nonce: String,
    #[serde(default)]
    pub signature: String,
    pub after: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ChannelAuth {
    #[serde(default)]
    pub ts: String,
    #[serde(default)]
    pub nonce: String,
    #[serde(default)]
    pub signature: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct MessageHistoryItem {
    pub id: String,
    pub channel: String,
    pub title: String,
    pub content: String,
    pub extras: String,
    pub delivery_status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct RealtimeMessage {
    pub id: String,
    pub channel: String,
    pub title: String,
    pub content: String,
    pub payload: serde_json::Value,
    pub kind: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct EncryptedEnvelope {
    pub version: u8,
    pub channel: String,
    pub algorithm: String,
    pub encrypted: bool,
    pub data: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateChannelRequest {
    pub name: String,
    #[serde(default)]
    pub key: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ChannelItem {
    pub id: String,
    pub name: String,
    pub key: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub fn default_channel() -> String {
    "default".to_owned()
}
