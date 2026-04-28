use crate::prelude::*;
use actix_multipart::Multipart;
use actix_web::{Error, HttpResponse, Responder, get, post, web};
use chrono::NaiveDateTime;
use futures_util::StreamExt;
use sqlx::{FromRow, PgPool};
use std::path::Path;
use tokio::{fs, io::AsyncWriteExt};
use uuid::Uuid;

//
// ✅ QUERY PARAM
//
#[derive(Deserialize)]
pub struct FileQuery {
    pub user_id: i32,
}

//
// ✅ RESPONSE STRUCT
//
#[derive(Serialize)]
struct FileResponse {
    id: i64,
    name: String,
    file_type: String,
    size: i64,
    drive_url: String,
    created_at: NaiveDateTime,
}

//
// ✅ DB STRUCT
//
#[derive(Serialize, FromRow)]
pub struct FileRecord {
    pub id: i64,
    pub name: String,
    pub file_path: String,
    pub size: i64,
    pub created_at: NaiveDateTime,
}

//
// 🔥 UPDATED UPLOAD FILE (FIXED USER_ID)
//
#[post("/files/upload")]
pub async fn upload_file(
    mut payload: Multipart,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let upload_dir = "./uploads";

    fs::create_dir_all(upload_dir).await.map_err(|e| {
        println!("❌ Dir error: {:?}", e);
        actix_web::error::ErrorInternalServerError("Dir error")
    })?;

    let mut user_id: i32 = 0;

    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|_| actix_web::error::ErrorBadRequest("Invalid multipart"))?;

        let field_name = field.name().to_string();

        // ✅ USER_ID
        if field_name == "user_id" {
            let mut bytes = Vec::new();

            while let Some(chunk) = field.next().await {
                let data =
                    chunk.map_err(|_| actix_web::error::ErrorBadRequest("User ID read error"))?;
                bytes.extend_from_slice(&data);
            }

            user_id = String::from_utf8(bytes)
                .ok()
                .and_then(|s| s.parse::<i32>().ok())
                .ok_or_else(|| actix_web::error::ErrorBadRequest("Invalid user_id"))?;

            continue;
        }

        // ✅ FILES ONLY
        if field_name != "files" {
            continue;
        }

        let content_disposition = field.content_disposition();

        let raw_filename = content_disposition
            .get_filename()
            .ok_or_else(|| actix_web::error::ErrorBadRequest("Missing filename"))?;

        // ✅ sanitize
        let filename = raw_filename.replace(['/', '\\'], "");

        let file_id = Uuid::new_v4().to_string();
        let filepath = format!("{}/{}_{}", upload_dir, file_id, filename);

        let mut f = fs::File::create(&filepath).await.map_err(|e| {
            println!("❌ File create error: {:?}", e);
            actix_web::error::ErrorInternalServerError("File create error")
        })?;

        let mut size: i64 = 0;

        while let Some(chunk) = field.next().await {
            let data = chunk.map_err(|_| actix_web::error::ErrorBadRequest("Chunk error"))?;

            size += data.len() as i64;

            f.write_all(&data).await.map_err(|e| {
                println!("❌ Write error: {:?}", e);
                actix_web::error::ErrorInternalServerError("Write error")
            })?;
        }

        // ✅ better file type extraction
        let file_type = filename.rsplit('.').next().unwrap_or("").to_string();

        if user_id == 0 {
            return Err(actix_web::error::ErrorBadRequest("Missing user_id"));
        }

        sqlx::query(
            r#"
            INSERT INTO files (name, file_path, size, file_type, user_id)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(&filename)
        .bind(&filepath)
        .bind(size)
        .bind(&file_type)
        .bind(user_id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| {
            println!("❌ DB Insert Error: {:?}", e);
            actix_web::error::ErrorInternalServerError("DB error")
        })?;
    }

    Ok(HttpResponse::Ok().body("Upload successful"))
}

//
// ✅ GET FILES
//
#[get("/files")]
pub async fn get_files(pool: web::Data<PgPool>, query: web::Query<FileQuery>) -> impl Responder {
    println!("📥 Fetch files for user_id: {}", query.user_id);

    let result = sqlx::query_as::<_, FileRecord>(
        "SELECT id, name, file_path, size, created_at FROM files WHERE user_id = $1 ORDER BY created_at DESC"
    )
    .bind(query.user_id)
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(rows) => {
            println!("📦 Found {} files", rows.len());

            let files: Vec<FileResponse> = rows
                .into_iter()
                .map(|row| {
                    let file_name = Path::new(&row.file_path)
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string();

                    let file_type = file_name.split('.').next_back().unwrap_or("").to_string();

                    FileResponse {
                        id: row.id,
                        name: row.name,
                        file_type,
                        size: row.size,
                        drive_url: format!("/uploads/{}", file_name),
                        created_at: row.created_at,
                    }
                })
                .collect();

            HttpResponse::Ok().json(files)
        }

        Err(e) => {
            println!("❌ DB error: {:?}", e);

            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Failed to fetch files" }))
        }
    }
}
