use crate::prelude::*;

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