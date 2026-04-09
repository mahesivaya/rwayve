use actix::Message;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Message, Clone)]
#[rtype(result = "()")]
pub struct SignalMessage {
    pub r#type: String,
    pub to: i32,
    pub from: Option<i32>,
    pub sdp: Option<String>,
    pub candidate: Option<IceCandidate>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IceCandidate {
    pub candidate: String,
    pub sdpMid: Option<String>,
    pub sdpMLineIndex: Option<u16>,
    pub usernameFragment: Option<String>,
}