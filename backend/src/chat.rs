use crate::prelude::*;
use crate::security::encryption::{encrypt, decrypt};
use crate::models::message::{Message, MessageRow, ChatMessage};


pub struct ChatSession {
    pub pool: PgPool,
}

// ✅ WebSocket entry point
pub async fn chat_ws(
    req: HttpRequest,
    stream: web::Payload,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    ws::start(
        ChatSession {
            pool: pool.get_ref().clone(),
        },
        &req,
        stream,
    )
}

// ✅ Actor setup
impl Actor for ChatSession {
    type Context = ws::WebsocketContext<Self>;
}

// ✅ Handle messages
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ChatSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                println!("📩 Incoming: {}", text);

                let parsed: Result<ChatMessage, _> = serde_json::from_str(&text);
                println!("🔍 Parsed: {:?}", parsed);

                if let Ok(data) = parsed {
                    let pool = self.pool.clone();

                    // 🔐 ENCRYPT MESSAGE
                    let (iv, encrypted) = match encrypt(&data.content) {
                        Ok(res) => res,
                        Err(e) => {
                            println!("❌ Encryption error: {:?}", e);
                            return;
                        }
                    };

                    let sender_id = data.sender_id;
                    let receiver_id = data.receiver_id;

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
                            Ok(_) => println!("🔐✅ Encrypted message saved"),
                            Err(e) => println!("❌ DB INSERT ERROR: {:?}", e),
                        }
                    };

                    ctx.spawn(actix::fut::wrap_future(fut));
                }

                // ✅ Keep UI working (send plaintext back)
                ctx.text(text);
            }

            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),

            Ok(ws::Message::Close(_)) => ctx.stop(),

            _ => {}
        }
    }
}


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

                // 🔓 DECRYPT HERE
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