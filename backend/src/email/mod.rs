pub mod attachments;
mod body_handlers;
pub mod body_worker;
pub mod handler;
pub mod oauth;
mod oauth_flow;
mod profile;
mod send;
pub mod sender;
pub mod sync;
pub mod utils;

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(crate::routes::email::get_emails)
        .service(crate::routes::email::get_all_email_attachments)
        .service(crate::routes::email::get_email_attachments)
        .service(crate::routes::email::download_email_attachment)
        .service(handler::get_email_body)
        .service(handler::get_email_by_id)
        .service(handler::send)
        .service(handler::get_me)
        .service(handler::save_public_key);
}

pub fn public_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/gmail/login", web::get().to(handler::gmail_login))
        .route("/oauth/callback", web::get().to(handler::oauth_callback));
}
