use crate::prelude::*;
use crate::security::encryption::{encrypt, decrypt};
use crate::models::message::{Message, ChatMessage, MessageStatus};

use actix::{Actor, StreamHandler, Handler, Addr, Message as ActixMessage, ActorFutureExt};
use actix_web_actors::ws;
use actix_web_actors::ws::WebsocketContext;

use serde::Deserialize;
use sqlx::{Row, PgPool};

use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;

// ================= GLOBAL SESSIONS =================

lazy_static! {
    static ref SESSIONS: Mutex<HashMap<i32, Addr<ChatSession>>> =
        Mutex::new(HashMap::new());
}

// ================= WS MESSAGE =================

#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

// ================= CHAT SESSION =================

pub struct ChatSession {
    pub pool: PgPool,
    pub user_id: i32,
}

// ================= WS ENTRY =================

pub async fn chat_ws(
    req: HttpRequest,
    stream: web::Payload,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {

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

// ================= ACTOR =================

impl Actor for ChatSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("🟢 User connected: {}", self.user_id);
        SESSIONS.lock().unwrap().insert(self.user_id, ctx.address());
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        println!("🔴 User disconnected: {}", self.user_id);
        SESSIONS.lock().unwrap().remove(&self.user_id);
    }
}

// ================= RECEIVE WS =================

impl Handler<WsMessage> for ChatSession {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

// ================= MAIN WS LOGIC =================

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ChatSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {

        match msg {
            Ok(ws::Message::Text(text)) => {
                println!("📩 Incoming: {}", text);

                let parsed: Result<ChatMessage, _> = serde_json::from_str(&text);

                if let Ok(data) = parsed {

                    // ================= READ RECEIPT =================
                    if matches!(data.status, Some(MessageStatus::Read)) {
                        let pool = self.pool.clone();
                        let reader = data.sender_id;
                        let other = data.receiver_id;

                        actix::spawn(async move {
                            let _ = sqlx::query(
                                r#"
                                UPDATE messages
                                SET status = 'read'
                                WHERE receiver_id = $1 AND sender_id = $2
                                "#
                            )
                            .bind(reader)
                            .bind(other)
                            .execute(&pool)
                            .await;
                        });

                        return;
                    }

                    // ================= NORMAL MESSAGE =================

                    let pool = self.pool.clone();
                    let sender_id = data.sender_id;
                    let receiver_id = data.receiver_id;
                    let content = data.content.clone();

                    let (iv, encrypted) = match encrypt(&content) {
                        Ok(res) => res,
                        Err(e) => {
                            println!("❌ Encryption error: {:?}", e);
                            return;
                        }
                    };

                    let fut = async move {
                        sqlx::query(
                            r#"
                            INSERT INTO messages 
                            (sender_id, receiver_id, content_encrypted, content_iv, status)
                            VALUES ($1, $2, $3, $4, 'sent')
                            RETURNING id
                            "#
                        )
                        .bind(sender_id)
                        .bind(receiver_id)
                        .bind(encrypted)
                        .bind(iv)
                        .fetch_one(&pool)
                        .await
                    };

                    let sender_addr = ctx.address();

                    ctx.spawn(
                        actix::fut::wrap_future(fut).map(
                            move |res, _act, ctx: &mut WebsocketContext<Self>| {

                                if let Ok(row) = res {
                                    let message_id: i32 = row.get("id");

                                    let msg_json = serde_json::json!({
                                        "message_id": message_id,
                                        "sender_id": sender_id,
                                        "receiver_id": receiver_id,
                                        "content": content,
                                        "status": "sent"
                                    })
                                    .to_string();

                                    // SEND TO RECEIVER
                                    if let Some(addr) = SESSIONS.lock().unwrap().get(&receiver_id) {
                                        addr.do_send(WsMessage(msg_json.clone()));

                                        // 🔥 DELIVERED
                                        let delivered_json = serde_json::json!({
                                            "type": "status_update",
                                            "message_id": message_id,
                                            "status": "delivered"
                                        })
                                        .to_string();

                                        sender_addr.do_send(WsMessage(delivered_json));
                                    }

                                    // SEND BACK TO SENDER
                                    ctx.text(msg_json);
                                }
                            },
                        )
                    );
                }
            }

            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Close(_)) => ctx.stop(),
            _ => {}
        }
    }
}

// ================= FETCH API =================

#[derive(Deserialize)]
pub struct QueryParams {
    pub user1: i32,
    pub user2: i32,
}

#[get("/messages")]
pub async fn get_messages(
    pool: web::Data<PgPool>,
    query: web::Query<QueryParams>,
) -> impl Responder {

    let result = sqlx::query(
        r#"
        SELECT id, sender_id, receiver_id, content_encrypted, content_iv, status, created_at
        FROM messages
        WHERE 
            (sender_id = $1 AND receiver_id = $2)
            OR
            (sender_id = $2 AND receiver_id = $1)
        ORDER BY created_at DESC
        LIMIT 50
        "#
    )
    .bind(query.user1)
    .bind(query.user2)
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(rows) => {

            let _ = sqlx::query(
                r#"
                UPDATE messages
                SET status = 'read'
                WHERE receiver_id = $1 AND sender_id = $2
                "#
            )
            .bind(query.user1)
            .bind(query.user2)
            .execute(pool.get_ref())
            .await;

            let mut messages: Vec<Message> = rows.into_iter().map(|row| {
                let encrypted: String = row.get("content_encrypted");
                let iv: String = row.get("content_iv");

                let content = decrypt(&iv, &encrypted);

                Message {
                    message_id: Some(row.get("id")),
                    sender_id: row.get("sender_id"),
                    receiver_id: row.get("receiver_id"),
                    content,
                    status: Some(row.get::<String, _>("status")),
                }
            }).collect();

            messages.reverse();

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