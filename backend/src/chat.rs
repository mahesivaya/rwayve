use crate::prelude::*;
use crate::security::encryption::{encrypt, decrypt};
use crate::models::message::{Message, ChatMessage};

use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;
use actix::{Actor, StreamHandler, Handler, Addr, Message as ActixMessage};
use actix_web::{web, HttpRequest, HttpResponse, Error, get};
use actix_web_actors::ws;
use futures_util::StreamExt;
use sqlx::{PgPool, Row};

//
// 🔥 GLOBAL SESSION STORE
//
lazy_static! {
    static ref SESSIONS: Mutex<HashMap<i32, Addr<ChatSession>>> =
        Mutex::new(HashMap::new());
}

//
// 🔥 WS MESSAGE TYPE
//
#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

//
// 🔥 CHAT SESSION
//
pub struct ChatSession {
    pub pool: PgPool,
    pub user_id: i32,
}

//
// 🔥 WS ENTRY POINT
//
pub async fn chat_ws(
    req: HttpRequest,
    stream: web::Payload,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {

    // ✅ extract user_id from query
    let user_id = req
        .query_string()
        .split('=')
        .nth(1)
        .and_then(|id| id.parse::<i32>().ok())
        .unwrap_or(0);

    ws::start(
        ChatSession {
            pool: pool.get_ref().clone(),
            user_id,
        },
        &req,
        stream,
    )
}

//
// 🔥 ACTOR LIFECYCLE
//
impl Actor for ChatSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("🟢 User connected: {}", self.user_id);

        SESSIONS
            .lock()
            .unwrap()
            .insert(self.user_id, ctx.address());
            println!("📡 Active sessions: {:?}", SESSIONS.lock().unwrap().keys());
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        println!("🔴 User disconnected: {}", self.user_id);

        SESSIONS.lock().unwrap().remove(&self.user_id);
        println!("📡 Active sessions: {:?}", SESSIONS.lock().unwrap().keys());

    }
}

//
// 🔥 HANDLE OUTGOING WS MESSAGE
//
impl Handler<WsMessage> for ChatSession {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

//
// 🔥 HANDLE INCOMING WS MESSAGE
//
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ChatSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {

        match msg {
            Ok(ws::Message::Text(text)) => {
                println!("📩 Incoming: {}", text);

                let parsed: Result<ChatMessage, _> = serde_json::from_str(&text);

                if let Ok(data) = parsed {

                    let pool = self.pool.clone();
                    let sender_id = data.sender_id;
                    let receiver_id = data.receiver_id;
                    let content = data.content.clone();

                    // 🔐 ENCRYPT
                    let (iv, encrypted) = match encrypt(&content) {
                        Ok(res) => res,
                        Err(e) => {
                            println!("❌ Encryption error: {:?}", e);
                            return;
                        }
                    };

                    // 💾 SAVE TO DB
                    let fut = async move {
                        match sqlx::query(
                            "INSERT INTO messages (sender_id, receiver_id, content_encrypted, content_iv)
                             VALUES ($1, $2, $3, $4)"
                        )
                        .bind(sender_id)
                        .bind(receiver_id)
                        .bind(encrypted)
                        .bind(iv)
                        .execute(&pool)
                        .await
                        {
                            Ok(_) => println!("✅ Message saved"),
                            Err(e) => println!("❌ DB ERROR: {:?}", e),
                        }
                    };

                    ctx.spawn(actix::fut::wrap_future(fut));

                    // 📦 CREATE JSON MESSAGE
                    let msg_json = serde_json::json!({
                        "sender_id": sender_id,
                        "receiver_id": receiver_id,
                        "content": content
                    })
                    .to_string();

                    // ✅ SEND TO RECEIVER (REAL-TIME)
                    if let Some(addr) = SESSIONS.lock().unwrap().get(&receiver_id) {
                        addr.do_send(WsMessage(msg_json.clone()));
                    }

                    // ✅ SEND BACK TO SENDER
                    ctx.text(msg_json);
                }
            }

            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),

            Ok(ws::Message::Close(_)) => ctx.stop(),

            _ => {}
        }
    }
}

//
// 🔥 FETCH MESSAGES API (DECRYPTED)
//
#[derive(Deserialize)]
pub struct QueryParams {
    pub user1: i32,
    pub user2: i32,
}

#[get("/api/messages")]
pub async fn get_messages(
    pool: web::Data<PgPool>,
    query: web::Query<QueryParams>,
) -> impl Responder {

    let result = sqlx::query(
        r#"
        SELECT sender_id, receiver_id, content_encrypted, content_iv
        FROM messages
        WHERE 
            (sender_id = $1 AND receiver_id = $2)
            OR
            (sender_id = $2 AND receiver_id = $1)
        ORDER BY id ASC
        "#
    )
    .bind(query.user1)
    .bind(query.user2)
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(rows) => {
            let messages: Vec<Message> = rows.into_iter().map(|row| {
                let encrypted: String = row.get("content_encrypted");
                let iv: String = row.get("content_iv");

                let content = decrypt(&iv, &encrypted);

                Message {
                    sender_id: row.get("sender_id"),
                    receiver_id: row.get("receiver_id"),
                    content,
                }
            }).collect();

            HttpResponse::Ok().json(messages)
        }

        Err(e) => {
            println!("❌ DB error: {:?}", e);

            HttpResponse::InternalServerError().json(
                serde_json::json!({ "error": "Failed to fetch messages" })
            )
        }
    }
}