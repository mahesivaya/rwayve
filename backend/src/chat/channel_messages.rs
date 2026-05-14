use crate::prelude::*;
use crate::security::encryption::decrypt;
use crate::security::jwt::get_user_id_from_request;

use super::dto::ChannelMessagesQuery;

use sqlx::Row;
use tracing::{error, instrument};

#[get("/chat/channel-messages")]
#[instrument(target = "http", skip(req, pool, query), fields(channel_id = query.channel_id))]
pub async fn get_channel_messages(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    query: web::Query<ChannelMessagesQuery>,
) -> impl Responder {
    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let is_member = match sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM channel_members
            WHERE channel_id = $1 AND user_id = $2
        )
        "#,
    )
    .bind(query.channel_id)
    .bind(user_id)
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(is_member) => is_member,
        Err(e) => {
            error!(target: "db", error = ?e, "get_channel_messages membership failed");
            return HttpResponse::InternalServerError().finish();
        }
    };

    if !is_member {
        return HttpResponse::Forbidden().finish();
    }

    let result = sqlx::query(
        r#"
        SELECT id, channel_id, sender_id, content_encrypted, content_iv, created_at
        FROM channel_messages
        WHERE channel_id = $1
        ORDER BY created_at DESC
        LIMIT 50
        "#,
    )
    .bind(query.channel_id)
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(rows) => {
            let mut messages: Vec<_> = rows
                .into_iter()
                .map(|row| {
                    let encrypted: String = row.get("content_encrypted");
                    let iv: String = row.get("content_iv");
                    let content = match decrypt(&iv, &encrypted) {
                        Ok(text) => text,
                        Err(e) => {
                            error!(target: "ws", error = %e, "channel message decrypt failed");
                            "[decryption failed]".to_string()
                        }
                    };

                    let created_naive: chrono::NaiveDateTime = row.get("created_at");
                    let created_at = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                        created_naive,
                        chrono::Utc,
                    );

                    serde_json::json!({
                        "message_id": row.get::<i32, _>("id"),
                        "channel_id": row.get::<i32, _>("channel_id"),
                        "sender_id": row.get::<i32, _>("sender_id"),
                        "content": content,
                        "status": "sent",
                        "created_at": created_at.to_rfc3339()
                    })
                })
                .collect();

            messages.reverse();
            HttpResponse::Ok().json(messages)
        }
        Err(e) => {
            error!(target: "db", error = ?e, "get_channel_messages query failed");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch channel messages"
            }))
        }
    }
}
