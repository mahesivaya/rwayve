use crate::prelude::*;


#[derive(serde::Serialize, FromRow)]
struct Account {
    id: i32,
    email: String,
}

#[get("/api/accounts")]
async fn get_accounts(pool: web::Data<PgPool>) -> impl Responder {
    let result = sqlx::query_as::<_, Account>(
        r#"
        SELECT id, email FROM email_accounts
        LIMIT 50
        "#
    )
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