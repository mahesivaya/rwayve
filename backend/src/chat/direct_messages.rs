use crate::cache::{Cache, chat_history_key};
use crate::models::message::Message;
use crate::prelude::*;
use crate::security::encryption::decrypt;
use crate::security::jwt::get_user_id_from_request;

use super::dto::QueryParams;

use sqlx::Row;
use tracing::{debug, error, instrument, warn};

#[get("/messages")]
#[instrument(target = "http", skip(req, pool, cache, query), fields(user1 = query.user1, user2 = query.user2))]
pub async fn get_messages(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    cache: web::Data<Option<Cache>>,
    query: web::Query<QueryParams>,
) -> impl Responder {
    // Auth: require a valid JWT and confirm the caller is one of the two
    // participants. Without this, any caller could read any conversation by
    // supplying arbitrary user1/user2 ids.
    let caller_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };

    if caller_id != query.user1 && caller_id != query.user2 {
        warn!(
            target: "auth",
            caller_id,
            user1 = query.user1,
            user2 = query.user2,
            "get_messages rejected: caller is not a conversation participant"
        );
        return HttpResponse::Forbidden().finish();
    }

    let cache_key = chat_history_key(query.user1, query.user2);

    if let Some(c) = cache.get_ref().as_ref()
        && let Some(cached) = c.get_json::<Vec<Message>>(&cache_key).await
    {
        debug!(target: "cache", key = %cache_key, "messages cache hit");
        // Still flip unread → read on every fetch so the sender sees the
        // status change even on cache hits.
        let _ = sqlx::query(
            "UPDATE messages
             SET status = 'read'
             WHERE receiver_id = $1 AND sender_id = $2 AND status <> 'read'",
        )
        .bind(query.user1)
        .bind(query.user2)
        .execute(pool.get_ref())
        .await;
        return HttpResponse::Ok().json(cached);
    }

    // Two ordered scans (each index-served by idx_messages_conversation /
    // idx_messages_reverse) merged via UNION ALL, then a final 50-row cap.
    // Faster than a single OR-predicate which forces a bitmap scan + sort.
    let result = sqlx::query(
        r#"
        SELECT id, sender_id, receiver_id, content_encrypted, content_iv, status::TEXT AS status, created_at
        FROM (
            (
                SELECT id, sender_id, receiver_id, content_encrypted, content_iv, status, created_at
                FROM messages
                WHERE sender_id = $1 AND receiver_id = $2
                ORDER BY created_at DESC
                LIMIT 50
            )
            UNION ALL
            (
                SELECT id, sender_id, receiver_id, content_encrypted, content_iv, status, created_at
                FROM messages
                WHERE sender_id = $2 AND receiver_id = $1
                ORDER BY created_at DESC
                LIMIT 50
            )
        ) AS m
        ORDER BY created_at DESC
        LIMIT 50
        "#
    )
    .bind(query.user1)
    .bind(query.user2)
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(rows) => {
            let _ = sqlx::query(
                r#"
                UPDATE messages
                SET status = 'read'
                WHERE receiver_id = $1 AND sender_id = $2
                  AND status <> 'read'
                "#,
            )
            .bind(query.user1)
            .bind(query.user2)
            .execute(pool.get_ref())
            .await;

            let mut messages: Vec<Message> = rows
                .into_iter()
                .map(|row| {
                    let encrypted: String = row.get("content_encrypted");
                    let iv: String = row.get("content_iv");

                    let content = match decrypt(&iv, &encrypted) {
                        Ok(text) => text,
                        Err(e) => {
                            error!(target: "ws", error = %e, "message decrypt failed");
                            "[decryption failed]".to_string()
                        }
                    };

                    let created_naive: chrono::NaiveDateTime = row.get("created_at");
                    let created_at = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                        created_naive,
                        chrono::Utc,
                    );

                    Message {
                        message_id: Some(row.get("id")),
                        sender_id: row.get("sender_id"),
                        receiver_id: row.get("receiver_id"),
                        content,
                        status: Some(row.get::<String, _>("status")),
                        created_at: Some(created_at),
                    }
                })
                .collect();

            messages.reverse();

            if let Some(c) = cache.get_ref().as_ref() {
                c.set_json_with_ttl(&cache_key, &messages, 60).await;
            }

            HttpResponse::Ok().json(messages)
        }

        Err(e) => {
            error!(target: "db", error = ?e, "get_messages query failed");

            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Failed to fetch messages" }))
        }
    }
}
