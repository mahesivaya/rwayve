use crate::prelude::*;
use sqlx::Type;


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Type)]
#[serde(rename_all = "lowercase")]
pub enum MessageStatus {
    Sent,
    Delivered,
    Read,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub message_id: Option<i32>,
    pub sender_id: i32,
    pub receiver_id: i32,
    pub content: String,
    pub status: Option<String>,
}
//
#[derive(Serialize)]
pub struct MessageResponse {
    pub message: String,
}

//

#[derive(Serialize, Deserialize)]
pub struct ChatMessage {
    pub sender_id: i32,
    pub receiver_id: i32,
    pub content: String,
    pub status: Option<MessageStatus>,
    pub message_id: Option<i32>,
}
