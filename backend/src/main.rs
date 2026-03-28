use actix_web::{get, web, App, HttpServer, HttpResponse};
use sqlx::PgPool;
use std::env;
use tokio::time::{sleep, Duration};
use actix_cors::Cors;


mod gmail;
use crate::gmail::{gmail_login, oauth_callback};


use serde::Serialize;
use sqlx::FromRow;

#[derive(Serialize, FromRow)]
struct Email {
    id: i32,
    sender: String,
    subject: String,
    body: String,
    created_at: chrono::NaiveDateTime,
}


#[get("/")]
async fn index() -> HttpResponse {
    HttpResponse::Ok().body("Email Import Running")
}


use actix_web::{Responder};

async fn get_emails(pool: web::Data<PgPool>) -> impl Responder {
    let result = sqlx::query_as::<_, Email>(
        r#"
        SELECT id, sender, subject, body, created_at
        FROM emails
        ORDER BY created_at DESC
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


#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let db_url =
        env::var("DATABASE_URL")
            .expect("DATABASE_URL missing");

    let pool =
        PgPool::connect(&db_url)
            .await
            .expect("DB failed");

    let pool_clone = pool.clone();

    // ✅ background sync worker
    tokio::spawn(async move {

        loop {

            println!("Sync loop");

            if let Err(e) =
                gmail::sync_all(&pool_clone).await
            {
                println!("sync error {:?}", e);
            }

            sleep(Duration::from_secs(60)).await;
        }
    });

    // ✅ Actix server
    HttpServer::new(move || {

        let cors = Cors::default()
            .allowed_origin("http://localhost:3000") // React dev server
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![
                actix_web::http::header::CONTENT_TYPE,
                actix_web::http::header::AUTHORIZATION,
            ])
            .supports_credentials();
            
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(index)
            .route("/gmail/login", web::get().to(gmail_login))
            .route("/oauth/callback", web::get().to(oauth_callback))
            .route("/emails", web::get().to(get_emails))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}