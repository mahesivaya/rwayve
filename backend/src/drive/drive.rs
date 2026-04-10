use crate::prelude::*;
use actix_multipart::Multipart;
use actix_web::{get, post, web, HttpResponse, Responder};
use futures_util::StreamExt;
use sqlx::{PgPool, FromRow};
use uuid::Uuid;
use chrono::NaiveDateTime;
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;

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
#[post("/api/files/upload")]
pub async fn upload_file(
    mut payload: Multipart,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let upload_dir = "./uploads";

    if let Err(e) = fs::create_dir_all(upload_dir).await {
        println!("❌ Dir error: {:?}", e);
        return HttpResponse::InternalServerError().finish();
    }

    let mut user_id: i32 = 0;

    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(f) => f,
            Err(_) => return HttpResponse::BadRequest().body("Invalid multipart"),
        };

        let field_name = field.name().to_string();

        //
        // ✅ EXTRACT USER_ID
        //
        if field_name == "user_id" {
            let mut bytes = Vec::new();

            while let Some(chunk) = field.next().await {
                let data = match chunk {
                    Ok(d) => d,
                    Err(_) => return HttpResponse::BadRequest().body("User ID read error"),
                };
                bytes.extend_from_slice(&data);
            }

            user_id = match String::from_utf8(bytes)
                .ok()
                .and_then(|s| s.parse::<i32>().ok())
            {
                Some(id) => id,
                None => return HttpResponse::BadRequest().body("Invalid user_id"),
            };

            println!("👤 Upload user_id: {}", user_id);
            continue;
        }

        //
        // ✅ ONLY PROCESS FILES
        //
        if field_name != "files" {
            continue;
        }

        let content_disposition = field.content_disposition();

        let raw_filename = match content_disposition.get_filename() {
            Some(name) => name,
            None => continue,
        };

        //
        // ✅ SANITIZE filename
        //
        let filename = raw_filename.replace("/", "").replace("\\", "");

        let file_id = Uuid::new_v4().to_string();
        let filepath = format!("{}/{}_{}", upload_dir, file_id, filename);

        let mut f = match fs::File::create(&filepath).await {
            Ok(file) => file,
            Err(e) => {
                println!("❌ File create error: {:?}", e);
                return HttpResponse::InternalServerError().finish();
            }
        };

        let mut size: i64 = 0;

        while let Some(chunk) = field.next().await {
            let data = match chunk {
                Ok(d) => d,
                Err(_) => return HttpResponse::BadRequest().body("Chunk error"),
            };

            size += data.len() as i64;

            if let Err(e) = f.write_all(&data).await {
                println!("❌ Write error: {:?}", e);
                return HttpResponse::InternalServerError().finish();
            }
        }

        let file_type = filename.split('.').last().unwrap_or("").to_string();

        //
        // 🚨 ENSURE USER_ID EXISTS
        //
        if user_id == 0 {
            return HttpResponse::BadRequest().body("Missing user_id");
        }

        //
        // ✅ INSERT INTO DB
        //
        if let Err(e) = sqlx::query(
            r#"
            INSERT INTO files (name, file_path, size, file_type, user_id)
            VALUES ($1, $2, $3, $4, $5)
            "#
        )
        .bind(&filename)
        .bind(&filepath)
        .bind(size)
        .bind(&file_type)
        .bind(user_id)
        .execute(pool.get_ref())
        .await
        {
            println!("❌ DB Insert Error: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }

        println!("✅ File saved: {} for user {}", filename, user_id);
    }

    HttpResponse::Ok().body("Upload successful")
}

//
// ✅ GET FILES
//
#[get("/api/files")]
pub async fn get_files(
    pool: web::Data<PgPool>,
    query: web::Query<FileQuery>,
) -> impl Responder {
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

                    let file_type = file_name
                        .split('.')
                        .last()
                        .unwrap_or("")
                        .to_string();

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

            HttpResponse::InternalServerError().json(
                serde_json::json!({ "error": "Failed to fetch files" })
            )
        }
    }
}