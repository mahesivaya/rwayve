use crate::prelude::*;
use crate::services::gmail_service;
use actix_web::{get, HttpResponse, Responder};


#[get("/gmail/login")]
pub async fn gmail_login() -> impl Responder {
    let url = gmail_service::build_gmail_oauth_url();
    HttpResponse::Found()
        .append_header(("Location", url))
        .finish()
}