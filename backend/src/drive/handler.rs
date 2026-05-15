use crate::prelude::*;
use crate::security::jwt::get_user_id_from_request;
use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::{Error, HttpResponse, Responder, get, post, web};
use chrono::NaiveDateTime;
use futures_util::StreamExt;
use sqlx::{FromRow, PgPool, Row};
use std::path::Path;
use tokio::{fs, io::AsyncWriteExt};
use tracing::{debug, error, info, instrument};
use uuid::Uuid;

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
#[instrument(target = "http", skip(req, payload, pool))]
pub async fn upload_file(
    req: HttpRequest,
    mut payload: Multipart,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    // Owner is derived from the verified JWT, never from the request body —
    // a `user_id` form field (if any) is ignored.
    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return Ok(HttpResponse::Unauthorized().finish()),
    };

    let upload_dir = "./uploads";

    fs::create_dir_all(upload_dir).await.map_err(|e| {
        error!(target: "http", error = ?e, "upload dir create failed");
        actix_web::error::ErrorInternalServerError("Dir error")
    })?;

    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|_| actix_web::error::ErrorBadRequest("Invalid multipart"))?;

        let field_name = field.name().to_string();

        // ✅ FILES ONLY (any other field, e.g. a stray user_id, is skipped)
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
            error!(target: "http", path = %filepath, error = ?e, "file create failed");
            actix_web::error::ErrorInternalServerError("File create error")
        })?;

        let mut size: i64 = 0;

        while let Some(chunk) = field.next().await {
            let data = chunk.map_err(|_| actix_web::error::ErrorBadRequest("Chunk error"))?;

            size += data.len() as i64;

            f.write_all(&data).await.map_err(|e| {
                error!(target: "http", path = %filepath, error = ?e, "file write failed");
                actix_web::error::ErrorInternalServerError("Write error")
            })?;
        }

        // ✅ better file type extraction
        let file_type = filename.rsplit('.').next().unwrap_or("").to_string();

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
            error!("Files insert failed (user_id={}): {:?}", user_id, e);
            actix_web::error::ErrorInternalServerError("DB error")
        })?;

        info!(
            "File uploaded: name=\"{}\" size={} user_id={}",
            filename, size, user_id
        );
    }

    Ok(HttpResponse::Ok().body("Upload successful"))
}

//
// ✅ GET FILES
//
#[get("/files")]
#[instrument(target = "http", skip(req, pool))]
pub async fn get_files(req: HttpRequest, pool: web::Data<PgPool>) -> impl Responder {
    // Files are scoped to the authenticated user — the previous `?user_id=`
    // query param let any caller list anyone's files.
    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let result = sqlx::query_as::<_, FileRecord>(
        "SELECT id, name, file_path, size, created_at FROM files WHERE user_id = $1 ORDER BY created_at DESC"
    )
    .bind(user_id)
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(rows) => {
            debug!(target: "http", user_id, count = rows.len(), "files listed");

            let files: Vec<FileResponse> = rows
                .into_iter()
                .map(|row| {
                    let file_name = Path::new(&row.file_path)
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default();

                    let file_type = file_name.split('.').next_back().unwrap_or("").to_string();

                    FileResponse {
                        id: row.id,
                        name: row.name,
                        file_type,
                        size: row.size,
                        // Authenticated, ownership-checked download route.
                        drive_url: format!("/api/files/{}/download", row.id),
                        created_at: row.created_at,
                    }
                })
                .collect();

            HttpResponse::Ok().json(files)
        }

        Err(e) => {
            error!(target: "db", user_id, error = ?e, "files list failed");

            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Failed to fetch files" }))
        }
    }
}

//
// 🔥 AUTHENTICATED DOWNLOAD
//
#[get("/files/{id}/download")]
#[instrument(target = "http", skip(req, pool, path))]
pub async fn download_file(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<i64>,
) -> impl Responder {
    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let file_id = path.into_inner();

    // Ownership check: the row is only returned when it belongs to the caller,
    // so a 404 leaks nothing about other users' files.
    let row = sqlx::query("SELECT file_path FROM files WHERE id = $1 AND user_id = $2")
        .bind(file_id)
        .bind(user_id)
        .fetch_optional(pool.get_ref())
        .await;

    let file_path: String = match row {
        Ok(Some(row)) => row.get("file_path"),
        Ok(None) => return HttpResponse::NotFound().finish(),
        Err(e) => {
            error!(target: "db", user_id, file_id, error = ?e, "download_file lookup failed");
            return HttpResponse::InternalServerError().finish();
        }
    };

    match NamedFile::open_async(&file_path).await {
        Ok(file) => file.into_response(&req),
        Err(e) => {
            error!(target: "http", user_id, file_id, error = ?e, "download_file open failed");
            HttpResponse::NotFound().finish()
        }
    }
}

#[cfg(test)]
mod auth_regression_tests {
    use super::*;
    use actix_web::{App, http::StatusCode, test, web};
    use sqlx::postgres::PgPoolOptions;

    fn lazy_pool() -> PgPool {
        PgPoolOptions::new()
            .connect_lazy("postgres://postgres:postgres@localhost/rwayve_test")
            .expect("lazy pool")
    }

    #[actix_web::test]
    async fn upload_file_requires_auth() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(lazy_pool()))
                .service(upload_file),
        )
        .await;

        let req = test::TestRequest::post().uri("/files/upload").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn get_files_requires_auth_even_with_user_id_query() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(lazy_pool()))
                .service(get_files),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/files?user_id=123")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn download_file_requires_auth() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(lazy_pool()))
                .service(download_file),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/files/1/download")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
