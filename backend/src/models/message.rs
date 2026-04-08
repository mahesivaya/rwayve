use serde::{Serialize, Deserialize};
use sqlx::FromRow;

//
// 🔹 1. DB STRUCT (Encrypted)
// Used internally when fetching from DB
//
#[derive(Debug, FromRow)]
pub struct MessageRow {
    pub sender_id: i32,
    pub receiver_id: i32,
    pub content_encrypted: String,
    pub content_iv: String,
}

//
// 🔹 2. API STRUCT (Decrypted → Sent to frontend)
//
#[derive(Debug, Serialize)]
pub struct Message {
    pub sender_id: i32,
    pub receiver_id: i32,
    pub content: String,
}

//
// 🔹 3. Incoming WebSocket / API Payload
//
#[derive(Debug, Deserialize)]
pub struct ChatMessage {
    pub sender_id: i32,
    pub receiver_id: i32,
    pub content: String,
}