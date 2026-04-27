use crate::models::account::Account;
use crate::prelude::*;

#[get("/accounts")]
async fn get_accounts(req: HttpRequest, pool: web::Data<PgPool>) -> impl Responder {
    // 🔥 Extract token
    let token = match req.headers().get("Authorization") {
        Some(h) => h.to_str().unwrap_or("").replace("Bearer ", ""),
        None => return HttpResponse::Unauthorized().body("Missing token"),
    };

    // 🔥 Decode JWT
    let decoded = match crate::models::auth::decode_jwt(&token) {
        Some(d) => d,
        None => return HttpResponse::Unauthorized().body("Invalid token"),
    };

    let user_id = decoded.sub;

    // ✅ Filter by user_id
    let result = sqlx::query_as::<_, Account>(
        r#"
        SELECT id, email
        FROM email_accounts
        WHERE user_id = $1
        ORDER BY id DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(rows) => HttpResponse::Ok().json(rows),
        Err(e) => {
            println!("DB error: {:?}", e);
            HttpResponse::InternalServerError().body("error")
        }
    }
}
