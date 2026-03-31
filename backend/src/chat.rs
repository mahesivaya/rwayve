use actix::{Actor, StreamHandler, ActorContext, AsyncContext};
use actix_web::{web, HttpRequest, HttpResponse, Error};
use actix_web_actors::ws;
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize, Debug)]
struct ChatMessage {
    sender_id: i32,
    receiver_id: i32,
    content: String,
}

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

                    let fut = async move {
                        match sqlx::query(
                            "INSERT INTO messages (sender_id, receiver_id, content) VALUES ($1, $2, $3)"
                        )
                        .bind(data.sender_id)
                        .bind(data.receiver_id)
                        .bind(data.content)
                        .execute(&pool)
                        .await
                        {
                            Ok(_) => println!("✅ Message saved to DB"),
                            Err(e) => println!("❌ DB INSERT ERROR: {:?}", e),
                        }
                    };

                    ctx.spawn(actix::fut::wrap_future(fut));
                }

                // echo back to UI
                ctx.text(text);
            }

            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),

            Ok(ws::Message::Close(_)) => ctx.stop(),

            _ => {}
        }
    }
}