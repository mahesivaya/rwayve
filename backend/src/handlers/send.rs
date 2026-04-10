use crate::prelude::*;
use crate::services::send_service;
use crate::models::email_request::SendEmailRequest;

#[post("/api/send")]
pub async fn send(
    data: web::Json<SendEmailRequest>,
    pool: web::Data<PgPool>,
) -> HttpResponse {

    // ✅ validation stays in handler
    if data.to.trim().is_empty() || data.subject.trim().is_empty() {
        return HttpResponse::BadRequest()
            .body("Recipient and Subject are required");
    }

    // ✅ call service
    match send_service::send_email(
        pool.get_ref(),
        data.account_id,
        data.to.clone(),
        data.subject.clone(),
        data.body.clone(),
    ).await {

        Ok(_) => HttpResponse::Ok().body("Email sent ✅"),

        Err(e) => {
            if e == "Email account not found" {
                HttpResponse::Unauthorized().body(e)
            } else {
                HttpResponse::InternalServerError().body(e)
            }
        }
    }
}