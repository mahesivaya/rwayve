use serde::{Serialize, Deserialize};
use actix::Message;

#[derive(Serialize, Deserialize, Message, Clone)]
#[rtype(result = "()")]
pub struct SignalMessage {
    pub r#type: String,
    pub to: i32,
    pub from: i32,
    pub sdp: Option<String>,
    pub candidate: Option<serde_json::Value>,
}