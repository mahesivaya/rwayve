pub mod handler;

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(handler::get_messages);
}

pub fn ws_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/ws/chat", web::get().to(handler::chat_ws));
}
