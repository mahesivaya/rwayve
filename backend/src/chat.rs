use actix::{Actor, StreamHandler, ActorContext};
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

impl Actor for ChatSession {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ChatSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                println!("Incoming: {}", text);

                let parsed: Result<ChatMessage, _> = serde_json::from_str(&text);

                if let Ok(data) = parsed {
                    let pool = self.pool.clone();

                    actix::spawn(async move {
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
                    });
                }

                ctx.text(text);
            }

            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),

            Ok(ws::Message::Close(_)) => ctx.stop(),

            _ => {}
        }
    }
}