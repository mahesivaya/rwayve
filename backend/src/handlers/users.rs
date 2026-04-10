use crate::prelude::*;


#[derive(serde::Serialize, sqlx::FromRow)]
pub struct UserResponse {
    id: i32,
    email: String,
}


#[get("/api/users")]
async fn get_users(pool: web::Data<PgPool>) -> impl Responder {
    let result = sqlx::query_as::<_, UserResponse>(
        r#"
        SELECT id, email FROM users
        "#
    )
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(rows) => HttpResponse::Ok().json(rows),
        Err(e) => {
            println!("DB error: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}