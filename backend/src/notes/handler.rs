use crate::models::note::{Note, NoteInput};
use crate::prelude::*;
use crate::security::jwt::get_user_id_from_request;
use actix_web::{delete, put};
use tracing::{error, instrument};

#[get("/notes")]
#[instrument(target = "http", skip(req, pool))]
pub async fn list_notes(req: HttpRequest, pool: web::Data<PgPool>) -> impl Responder {
    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let result = sqlx::query_as::<_, Note>(
        "SELECT id, title, content, created_at, updated_at
         FROM notes
         WHERE user_id = $1
         ORDER BY updated_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(rows) => HttpResponse::Ok().json(rows),
        Err(e) => {
            error!(target: "db", user_id, error = ?e, "notes list failed");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[post("/notes")]
#[instrument(target = "http", skip(req, pool, data))]
pub async fn create_note(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    data: web::Json<NoteInput>,
) -> impl Responder {
    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let result = sqlx::query_as::<_, Note>(
        "INSERT INTO notes (user_id, title, content)
         VALUES ($1, $2, $3)
         RETURNING id, title, content, created_at, updated_at",
    )
    .bind(user_id)
    .bind(data.title.as_deref().unwrap_or(""))
    .bind(data.content.as_deref().unwrap_or(""))
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(note) => HttpResponse::Ok().json(note),
        Err(e) => {
            error!(target: "db", user_id, error = ?e, "notes create failed");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[put("/notes/{id}")]
#[instrument(target = "http", skip(req, pool, path, data))]
pub async fn update_note(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
    data: web::Json<NoteInput>,
) -> impl Responder {
    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let id = path.into_inner();

    // Owner-scoped UPDATE — silently no-ops if the note belongs to someone
    // else, so we 404 rather than leaking that the id exists.
    let result = sqlx::query_as::<_, Note>(
        "UPDATE notes
         SET title = $1, content = $2, updated_at = NOW()
         WHERE id = $3 AND user_id = $4
         RETURNING id, title, content, created_at, updated_at",
    )
    .bind(data.title.as_deref().unwrap_or(""))
    .bind(data.content.as_deref().unwrap_or(""))
    .bind(id)
    .bind(user_id)
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(note)) => HttpResponse::Ok().json(note),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => {
            error!(target: "db", user_id, note_id = id, error = ?e, "notes update failed");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[delete("/notes/{id}")]
#[instrument(target = "http", skip(req, pool, path))]
pub async fn delete_note(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
) -> impl Responder {
    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let id = path.into_inner();

    let result = sqlx::query("DELETE FROM notes WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(r) if r.rows_affected() == 0 => HttpResponse::NotFound().finish(),
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({ "deleted": true })),
        Err(e) => {
            error!(target: "db", user_id, note_id = id, error = ?e, "notes delete failed");
            HttpResponse::InternalServerError().finish()
        }
    }
}
