use crate::prelude::*;
use crate::security::jwt::get_user_id_from_request;

use super::dto::{UpdateChannelSubjectInput, UpdateChannelVisibilityInput};
use super::helpers::is_channel_admin;

use actix_web::patch;
use tracing::{error, instrument};

#[patch("/chat/channels/{channel_id}")]
#[instrument(target = "http", skip(req, pool, input), fields(channel_id))]
pub async fn update_channel_subject(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    channel_id: web::Path<i32>,
    input: web::Json<UpdateChannelSubjectInput>,
) -> impl Responder {
    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };
    let channel_id = channel_id.into_inner();

    match is_channel_admin(pool.get_ref(), channel_id, user_id).await {
        Ok(true) => {}
        Ok(false) => {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "error": "Only channel admins can change the subject"
            }));
        }
        Err(e) => {
            error!(target: "db", error = ?e, channel_id, "update_channel_subject admin check failed");
            return HttpResponse::InternalServerError().finish();
        }
    }

    let name = input.name.trim();
    if name.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Channel subject is required"
        }));
    }

    match sqlx::query("UPDATE channels SET name = $1 WHERE id = $2")
        .bind(name)
        .bind(channel_id)
        .execute(pool.get_ref())
        .await
    {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({ "name": name })),
        Err(e) => {
            error!(target: "db", error = ?e, channel_id, "update_channel_subject failed");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[patch("/chat/channels/{channel_id}/visibility")]
#[instrument(target = "http", skip(req, pool, input), fields(channel_id))]
pub async fn update_channel_visibility(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    channel_id: web::Path<i32>,
    input: web::Json<UpdateChannelVisibilityInput>,
) -> impl Responder {
    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };
    let channel_id = channel_id.into_inner();

    match is_channel_admin(pool.get_ref(), channel_id, user_id).await {
        Ok(true) => {}
        Ok(false) => {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "error": "Only channel admins can change visibility"
            }));
        }
        Err(e) => {
            error!(target: "db", error = ?e, channel_id, "update_channel_visibility admin check failed");
            return HttpResponse::InternalServerError().finish();
        }
    }

    let visibility = match input.visibility.as_str() {
        "public" => "public",
        _ => "private",
    };

    match sqlx::query("UPDATE channels SET visibility = $1 WHERE id = $2")
        .bind(visibility)
        .bind(channel_id)
        .execute(pool.get_ref())
        .await
    {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({ "visibility": visibility })),
        Err(e) => {
            error!(target: "db", error = ?e, channel_id, "update_channel_visibility failed");
            HttpResponse::InternalServerError().finish()
        }
    }
}
