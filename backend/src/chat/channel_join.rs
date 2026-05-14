use crate::prelude::*;
use crate::security::jwt::get_user_id_from_request;

use super::dto::JoinRequestActionInput;
use super::helpers::is_channel_admin;

use sqlx::Row;
use tracing::{error, instrument};

#[post("/chat/channels/{channel_id}/join")]
#[instrument(target = "http", skip(req, pool), fields(channel_id))]
pub async fn join_channel(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    channel_id: web::Path<i32>,
) -> impl Responder {
    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };
    let channel_id = channel_id.into_inner();

    let row = match sqlx::query(
        r#"
        SELECT
            visibility,
            EXISTS(
                SELECT 1 FROM channel_members
                WHERE channel_id = channels.id AND user_id = $2
            ) AS is_member
        FROM channels
        WHERE id = $1
        "#,
    )
    .bind(channel_id)
    .bind(user_id)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(row)) => row,
        Ok(None) => return HttpResponse::NotFound().finish(),
        Err(e) => {
            error!(target: "db", error = ?e, channel_id, "join_channel lookup failed");
            return HttpResponse::InternalServerError().finish();
        }
    };

    if row.get::<bool, _>("is_member") {
        return HttpResponse::Ok().json(serde_json::json!({ "status": "joined" }));
    }

    if row.get::<String, _>("visibility") == "public" {
        match sqlx::query(
            r#"
            INSERT INTO channel_members (channel_id, user_id, role)
            VALUES ($1, $2, 'user')
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(channel_id)
        .bind(user_id)
        .execute(pool.get_ref())
        .await
        {
            Ok(_) => HttpResponse::Ok().json(serde_json::json!({ "status": "joined" })),
            Err(e) => {
                error!(target: "db", error = ?e, channel_id, user_id, "join_channel public join failed");
                HttpResponse::InternalServerError().finish()
            }
        }
    } else {
        match sqlx::query(
            r#"
            INSERT INTO channel_join_requests (channel_id, user_id, status)
            VALUES ($1, $2, 'pending')
            ON CONFLICT (channel_id, user_id)
            DO UPDATE SET status = 'pending', requested_at = NOW()
            "#,
        )
        .bind(channel_id)
        .bind(user_id)
        .execute(pool.get_ref())
        .await
        {
            Ok(_) => HttpResponse::Ok().json(serde_json::json!({ "status": "pending" })),
            Err(e) => {
                error!(target: "db", error = ?e, channel_id, user_id, "join_channel private request failed");
                HttpResponse::InternalServerError().finish()
            }
        }
    }
}

#[post("/chat/channels/{channel_id}/join-requests/approve")]
#[instrument(target = "http", skip(req, pool, input), fields(channel_id))]
pub async fn approve_channel_join_request(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    channel_id: web::Path<i32>,
    input: web::Json<JoinRequestActionInput>,
) -> impl Responder {
    let admin_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };
    let channel_id = channel_id.into_inner();

    match is_channel_admin(pool.get_ref(), channel_id, admin_id).await {
        Ok(true) => {}
        Ok(false) => {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "error": "Only channel admins can approve requests"
            }));
        }
        Err(e) => {
            error!(target: "db", error = ?e, channel_id, "approve_channel_join_request admin check failed");
            return HttpResponse::InternalServerError().finish();
        }
    }

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            error!(target: "db", error = ?e, channel_id, "approve_channel_join_request begin failed");
            return HttpResponse::InternalServerError().finish();
        }
    };

    if let Err(e) = sqlx::query(
        r#"
        INSERT INTO channel_members (channel_id, user_id, role)
        VALUES ($1, $2, 'user')
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(channel_id)
    .bind(input.user_id)
    .execute(&mut *tx)
    .await
    {
        error!(target: "db", error = ?e, channel_id, user_id = input.user_id, "approve_channel_join_request member insert failed");
        return HttpResponse::InternalServerError().finish();
    }

    if let Err(e) =
        sqlx::query("DELETE FROM channel_join_requests WHERE channel_id = $1 AND user_id = $2")
            .bind(channel_id)
            .bind(input.user_id)
            .execute(&mut *tx)
            .await
    {
        error!(target: "db", error = ?e, channel_id, user_id = input.user_id, "approve_channel_join_request cleanup failed");
        return HttpResponse::InternalServerError().finish();
    }

    if let Err(e) = tx.commit().await {
        error!(target: "db", error = ?e, channel_id, "approve_channel_join_request commit failed");
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}
