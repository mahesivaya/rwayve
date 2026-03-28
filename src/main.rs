use actix_web::{get, web, App, HttpServer, HttpResponse};
use sqlx::PgPool;
use std::env;
use tokio::time::{sleep, Duration};

mod gmail;
use crate::gmail::{gmail_login, oauth_callback};

#[get("/")]
async fn index() -> HttpResponse {
    HttpResponse::Ok().body("Email Import Running")
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
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(index)
            .route("/gmail/login", web::get().to(gmail_login))
            .route("/oauth/callback", web::get().to(oauth_callback))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}