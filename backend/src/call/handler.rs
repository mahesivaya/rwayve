use crate::prelude::*;
use actix::Addr;
use actix::*;
use actix_web_actors::ws;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;
use tracing::{debug, info, instrument, warn};

use crate::models::callmodel::SignalMessage;

lazy_static! {
    static ref SESSIONS: Mutex<HashMap<i32, Addr<CallSession>>> = Mutex::new(HashMap::new());
}

pub struct CallSession {
    pub user_id: i32,
}

impl Actor for CallSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Call WS connected: user_id={}", self.user_id);

        SESSIONS
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(self.user_id, ctx.address());
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        info!("Call WS disconnected: user_id={}", self.user_id);

        SESSIONS
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&self.user_id);
    }
}

impl Handler<SignalMessage> for CallSession {
    type Result = ();

    fn handle(&mut self, msg: SignalMessage, ctx: &mut Self::Context) {
        match serde_json::to_string(&msg) {
            Ok(text) => ctx.text(text),
            Err(e) => warn!(target: "ws", error = %e, "failed to serialize signal message"),
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for CallSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, _: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                debug!(target: "ws", user_id = self.user_id, len = text.len(), "call signal in");

                if let Ok(signal) = serde_json::from_str::<SignalMessage>(&text) {
                    let target = signal.to;

                    let sessions = SESSIONS.lock().unwrap_or_else(|e| e.into_inner());

                    if let Some(addr) = sessions.get(&target).cloned() {
                        debug!(target: "ws", from = self.user_id, to = target, kind = %signal.r#type, "forwarding signal");

                        addr.do_send(SignalMessage {
                            r#type: signal.r#type.clone(),
                            to: signal.to,
                            from: Some(self.user_id),
                            sdp: signal.sdp.clone(),
                            candidate: signal.candidate.clone(),
                        });
                    } else {
                        warn!(target: "ws", target_user = target, "signal target not connected");
                    }
                } else {
                    warn!(target: "ws", user_id = self.user_id, "failed to parse signal message");
                }
            }

            Ok(ws::Message::Close(_)) => {
                debug!(target: "ws", user_id = self.user_id, "call client closed");
            }

            _ => {}
        }
    }
}

#[instrument(target = "ws", skip(req, stream, query))]
pub async fn call_ws(
    req: HttpRequest,
    stream: web::Payload,
    query: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse, Error> {
    // Auth: prefer the httpOnly cookie, with query-token fallback for older clients.
    let token = match crate::security::jwt::token_from_request(&req)
        .or_else(|| query.get("token").cloned())
        .filter(|token| !token.trim().is_empty())
    {
        Some(token) => token,
        None => {
            warn!(target: "ws", "call_ws rejected: missing token");
            return Ok(HttpResponse::Unauthorized().body("Missing token"));
        }
    };

    let user_id = match crate::security::jwt::decode_jwt(&token) {
        Some(claims) => claims.sub,
        None => {
            warn!(target: "ws", "call_ws rejected: invalid token");
            return Ok(HttpResponse::Unauthorized().body("Invalid token"));
        }
    };

    info!("Call WS connect: user_id={}", user_id);

    ws::start(CallSession { user_id }, &req, stream)
}
