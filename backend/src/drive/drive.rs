use crate::prelude::*;
use actix_multipart::Multipart;
use actix_web::{get, post, web, HttpResponse, Responder};
use futures_util::StreamExt;
use sqlx::{PgPool, FromRow};
use std::fs;
use std::io::Write;
use uuid::Uuid;
use chrono::NaiveDateTime;


#[derive(Deserialize)]
pub struct FileQuery {
    pub user_id: i32,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct FileItem {
    pub id: i32,
    pub name: String,
    pub file_type: Option<String>,
    pub drive_url: Option<String>,
    pub size: i64,
    pub created_at: NaiveDateTime,
}


#[derive(serde::Serialize, FromRow)]
pub struct FileRecord {
    pub id: i64,
    pub name: String,
    pub file_path: String,
    pub size: i64,
    pub created_at: chrono::NaiveDateTime,
}

#[post("/api/files/upload")]
pub async fn upload_file(
    mut payload: Multipart,
    pool: web::Data<PgPool>,
) -> impl Responder {

    let upload_dir = "./uploads";
    fs::create_dir_all(upload_dir).unwrap();

    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(f) => f,
            Err(_) => return HttpResponse::BadRequest().body("Invalid multipart"),
        };

        let content_disposition = field.content_disposition();

        let filename = match content_disposition.get_filename() {
            Some(name) => name.to_string(),
            None => continue,
        };

        let file_id = Uuid::new_v4().to_string();
        let filepath = format!("{}/{}_{}", upload_dir, file_id, filename);

        let mut f = fs::File::create(&filepath).unwrap();
        let mut size: i64 = 0;

        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            size += data.len() as i64;
            f.write_all(&data).unwrap();
        }

        // ✅ Save to DB
        let _ = sqlx::query(
            r#"
            INSERT INTO files (user_id, name, file_path, size)
            VALUES ($1, $2, $3, $4)
            "#
        )
        .bind(&user_id)
        .bind(&filename)
        .bind(&filepath)
        .bind(size)
        .execute(pool.get_ref())
        .await;
    }

    HttpResponse::Ok().body("Upload successful")
}


#[get("/api/files")]
async fn get_files(
    pool: web::Data<PgPool>,
    query: web::Query<FileQuery>,
) -> impl Responder {

    let result = sqlx::query_as::<_, FileItem>(
        r#"
        SELECT id, name, file_type, size, drive_url, created_at
        FROM files
        WHERE user_id = $1 AND is_deleted = false
        ORDER BY created_at DESC
        "#
    )
    .bind(query.user_id)
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(files) => HttpResponse::Ok().json(files),
        Err(e) => {
            println!("DB error: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}