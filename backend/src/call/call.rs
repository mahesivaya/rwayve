use crate::prelude::*;
use actix::*;
use actix_web_actors::ws;
use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;
use actix::Addr;

use crate::models::callmodel::SignalMessage;

lazy_static! {
    static ref SESSIONS: Mutex<HashMap<i32, Addr<CallSession>>> =
        Mutex::new(HashMap::new());
}

pub struct CallSession {
    pub user_id: i32,
}

impl Actor for CallSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("✅ WS Connected: user {}", self.user_id);

        SESSIONS.lock().unwrap().insert(self.user_id, ctx.address());
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        println!("❌ WS Disconnected: user {}", self.user_id);

        SESSIONS.lock().unwrap().remove(&self.user_id);
    }
}

impl Handler<SignalMessage> for CallSession {
    type Result = ();

    fn handle(&mut self, msg: SignalMessage, ctx: &mut Self::Context) {
        ctx.text(serde_json::to_string(&msg).unwrap());
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for CallSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, _: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                println!("📩 Incoming: {}", text);

                if let Ok(signal) = serde_json::from_str::<SignalMessage>(&text) {

                    let target = signal.to;

                    let sessions = SESSIONS.lock().unwrap();

                    if let Some(addr) = sessions.get(&target).cloned() {
                        println!("➡️ Forwarding {} → {}", self.user_id, target);

                        addr.do_send(SignalMessage {
                            r#type: signal.r#type.clone(),
                            to: signal.to,
                            from: Some(self.user_id),
                            sdp: signal.sdp.clone(),
                            candidate: signal.candidate.clone(),
                        });

                    } else {
                        println!("⚠️ User {} not connected", target);
                    }

                } else {
                    println!("❌ Failed to parse message");
                }
            }

            Ok(ws::Message::Ping(_msg)) => {
                println!("🏓 Ping");
            }

            Ok(ws::Message::Close(_)) => {
                println!("🔌 Client closed connection");
            }

            _ => {}
        }
    }
}

pub async fn call_ws(
    req: HttpRequest,
    stream: web::Payload,
    query: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse, Error> {

    let user_id = query
        .get("user_id")
        .and_then(|id| id.parse::<i32>().ok())
        .unwrap_or(0);

    println!("🚀 WS CONNECT REQUEST user_id={}", user_id);

    ws::start(CallSession { user_id }, &req, stream)
}