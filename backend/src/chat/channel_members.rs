use crate::prelude::*;
use crate::security::jwt::get_user_id_from_request;

use super::dto::{AddChannelUsersInput, RemoveChannelUserInput};
use super::helpers::{is_channel_admin, normalize_channel_role, normalize_invite_emails};

use actix_web::delete;
use sqlx::Row;
use tracing::{error, instrument};

#[post("/chat/channels/{channel_id}/members")]
#[instrument(target = "http", skip(req, pool, input), fields(channel_id))]
pub async fn add_channel_users(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    channel_id: web::Path<i32>,
    input: web::Json<AddChannelUsersInput>,
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
                "error": "Only channel admins can add users"
            }));
        }
        Err(e) => {
            error!(target: "db", error = ?e, channel_id, "add_channel_users admin check failed");
            return HttpResponse::InternalServerError().finish();
        }
    }

    let invite_role = normalize_channel_role(input.invite_role.as_deref());
    let invite_emails = normalize_invite_emails(&input.invite_emails);
    if invite_emails.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Add at least one email"
        }));
    }

    let invited_users =
        match sqlx::query("SELECT id, email FROM users WHERE LOWER(email) = ANY($1)")
            .bind(&invite_emails)
            .fetch_all(pool.get_ref())
            .await
        {
            Ok(rows) => rows
                .into_iter()
                .map(|row| (row.get::<i32, _>("id"), row.get::<String, _>("email")))
                .collect::<Vec<_>>(),
            Err(e) => {
                error!(target: "db", error = ?e, channel_id, "add_channel_users lookup failed");
                return HttpResponse::InternalServerError().finish();
            }
        };

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            error!(target: "db", error = ?e, channel_id, "add_channel_users begin failed");
            return HttpResponse::InternalServerError().finish();
        }
    };

    for (member_id, email) in &invited_users {
        if let Err(e) = sqlx::query(
            r#"
            INSERT INTO channel_members (channel_id, user_id, role)
            VALUES ($1, $2, $3)
            ON CONFLICT (channel_id, user_id)
            DO UPDATE SET role = EXCLUDED.role
            "#,
        )
        .bind(channel_id)
        .bind(member_id)
        .bind(invite_role)
        .execute(&mut *tx)
        .await
        {
            error!(target: "db", error = ?e, channel_id, member_id, "add_channel_users member insert failed");
            return HttpResponse::InternalServerError().finish();
        }

        if let Err(e) = sqlx::query(
            "DELETE FROM channel_invites WHERE channel_id = $1 AND LOWER(email) = LOWER($2)",
        )
        .bind(channel_id)
        .bind(email)
        .execute(&mut *tx)
        .await
        {
            error!(target: "db", error = ?e, channel_id, email, "add_channel_users invite cleanup failed");
            return HttpResponse::InternalServerError().finish();
        }
    }

    let registered_invite_emails = invited_users
        .iter()
        .map(|(_id, email)| email.to_lowercase())
        .collect::<Vec<_>>();
    for email in invite_emails
        .iter()
        .filter(|email| !registered_invite_emails.contains(email))
    {
        if let Err(e) = sqlx::query(
            r#"
            INSERT INTO channel_invites (channel_id, email, role, invited_by)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (channel_id, email)
            DO UPDATE SET role = EXCLUDED.role, invited_by = EXCLUDED.invited_by
            "#,
        )
        .bind(channel_id)
        .bind(email)
        .bind(invite_role)
        .bind(user_id)
        .execute(&mut *tx)
        .await
        {
            error!(target: "db", error = ?e, channel_id, email, "add_channel_users invite insert failed");
            return HttpResponse::InternalServerError().finish();
        }
    }

    if let Err(e) = tx.commit().await {
        error!(target: "db", error = ?e, channel_id, "add_channel_users commit failed");
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

#[delete("/chat/channels/{channel_id}/members")]
#[instrument(target = "http", skip(req, pool, input), fields(channel_id))]
pub async fn remove_channel_user(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    channel_id: web::Path<i32>,
    input: web::Json<RemoveChannelUserInput>,
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
                "error": "Only channel admins can delete users"
            }));
        }
        Err(e) => {
            error!(target: "db", error = ?e, channel_id, "remove_channel_user admin check failed");
            return HttpResponse::InternalServerError().finish();
        }
    }

    let email = input.email.trim().to_lowercase();
    if email.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Email is required"
        }));
    }

    let target = match sqlx::query(
        r#"
        SELECT cm.user_id, cm.role
        FROM channel_members cm
        JOIN users u ON u.id = cm.user_id
        WHERE cm.channel_id = $1 AND LOWER(u.email) = $2
        "#,
    )
    .bind(channel_id)
    .bind(&email)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(target) => target,
        Err(e) => {
            error!(target: "db", error = ?e, channel_id, email, "remove_channel_user lookup failed");
            return HttpResponse::InternalServerError().finish();
        }
    };

    if let Some(row) = target {
        let target_user_id: i32 = row.get("user_id");
        let target_role: String = row.get("role");

        if target_role == "admin" {
            let admin_count = match sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM channel_members WHERE channel_id = $1 AND role = 'admin'",
            )
            .bind(channel_id)
            .fetch_one(pool.get_ref())
            .await
            {
                Ok(count) => count,
                Err(e) => {
                    error!(target: "db", error = ?e, channel_id, "remove_channel_user admin count failed");
                    return HttpResponse::InternalServerError().finish();
                }
            };

            if admin_count <= 1 {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "A channel must keep at least one admin"
                }));
            }
        }

        if let Err(e) =
            sqlx::query("DELETE FROM channel_members WHERE channel_id = $1 AND user_id = $2")
                .bind(channel_id)
                .bind(target_user_id)
                .execute(pool.get_ref())
                .await
        {
            error!(target: "db", error = ?e, channel_id, target_user_id, "remove_channel_user member delete failed");
            return HttpResponse::InternalServerError().finish();
        }
    } else if let Err(e) =
        sqlx::query("DELETE FROM channel_invites WHERE channel_id = $1 AND LOWER(email) = $2")
            .bind(channel_id)
            .bind(&email)
            .execute(pool.get_ref())
            .await
    {
        error!(target: "db", error = ?e, channel_id, email, "remove_channel_user invite delete failed");
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}
