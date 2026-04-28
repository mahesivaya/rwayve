use crate::prelude::*;
#[get("/health")]
pub async fn health() -> impl Responder {
    HttpResponse::Ok().body("ok")
}