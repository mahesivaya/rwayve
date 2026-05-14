use crate::prelude::*;
use crate::security::jwt::get_user_id_from_request;

use super::dto::CreateChannelInput;
use super::helpers::{normalize_channel_role, normalize_invite_emails};

use sqlx::Row;
use tracing::{error, instrument};

#[post("/chat/channels")]
#[instrument(target = "http", skip(req, pool, input))]
pub async fn create_channel(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    input: web::Json<CreateChannelInput>,
) -> impl Responder {
    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let name = input.name.trim();
    if name.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Channel name is required"
        }));
    }

    let invite_role = normalize_channel_role(input.invite_role.as_deref());
    let invite_emails = normalize_invite_emails(&input.invite_emails.clone().unwrap_or_default());

    let invited_users = if invite_emails.is_empty() {
        Vec::new()
    } else {
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
                error!(target: "db", error = ?e, "create_channel invite lookup failed");
                return HttpResponse::InternalServerError().finish();
            }
        }
    };

    let mut member_ids = input.member_ids.clone().unwrap_or_default();
    member_ids.extend(invited_users.iter().map(|(id, _email)| *id));
    member_ids.push(user_id);
    member_ids.sort_unstable();
    member_ids.dedup();

    if member_ids.len() < 2 && invite_emails.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Add at least one invitee email"
        }));
    }

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            error!(target: "db", error = ?e, "create_channel begin failed");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let row = match sqlx::query(
        r#"
        INSERT INTO channels (name, created_by)
        VALUES ($1, $2)
        RETURNING id, created_at
        "#,
    )
    .bind(name)
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(row) => row,
        Err(e) => {
            error!(target: "db", error = ?e, "create_channel insert failed");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let channel_id: i32 = row.get("id");
    for member_id in &member_ids {
        let role = if *member_id == user_id {
            "admin"
        } else {
            invite_role
        };

        if let Err(e) = sqlx::query(
            r#"
            INSERT INTO channel_members (channel_id, user_id, role)
            VALUES ($1, $2, $3)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(channel_id)
        .bind(member_id)
        .bind(role)
        .execute(&mut *tx)
        .await
        {
            error!(target: "db", error = ?e, channel_id, member_id, "create_channel member insert failed");
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "One or more members could not be added"
            }));
        }
    }

    let registered_invite_emails: Vec<String> = invited_users
        .iter()
        .map(|(_id, email)| email.to_lowercase())
        .collect();
    for email in invite_emails
        .iter()
        .filter(|email| !registered_invite_emails.contains(email))
    {
        if let Err(e) = sqlx::query(
            r#"
            INSERT INTO channel_invites (channel_id, email, role, invited_by)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(channel_id)
        .bind(email)
        .bind(invite_role)
        .bind(user_id)
        .execute(&mut *tx)
        .await
        {
            error!(target: "db", error = ?e, channel_id, email, "create_channel invite insert failed");
            return HttpResponse::InternalServerError().finish();
        }
    }

    if let Err(e) = tx.commit().await {
        error!(target: "db", error = ?e, channel_id, "create_channel commit failed");
        return HttpResponse::InternalServerError().finish();
    }

    let created_naive: chrono::NaiveDateTime = row.get("created_at");
    let created_at =
        chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(created_naive, chrono::Utc);
    let member_rows = match sqlx::query(
        r#"
        SELECT u.email, cm.role
        FROM channel_members cm
        JOIN users u ON u.id = cm.user_id
        WHERE cm.channel_id = $1
        ORDER BY u.email
        "#,
    )
    .bind(channel_id)
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            error!(target: "db", error = ?e, channel_id, "create_channel member email lookup failed");
            Vec::new()
        }
    };
    let mut member_emails = Vec::new();
    let mut admin_emails = Vec::new();
    let mut user_emails = Vec::new();
    for row in member_rows {
        let email: String = row.get("email");
        let role: String = row.get("role");
        member_emails.push(email.clone());
        if role == "admin" {
            admin_emails.push(email);
        } else {
            user_emails.push(email);
        }
    }

    let admin_invite_emails: Vec<String> = if invite_role == "admin" {
        invite_emails.clone()
    } else {
        Vec::new()
    };
    let user_invite_emails: Vec<String> = if invite_role == "user" {
        invite_emails.clone()
    } else {
        Vec::new()
    };

    HttpResponse::Created().json(serde_json::json!({
        "id": channel_id,
        "name": name,
        "visibility": "private",
        "created_by": user_id,
        "created_at": created_at.to_rfc3339(),
        "current_user_role": "admin",
        "is_member": true,
        "join_status": null,
        "member_ids": member_ids,
        "member_emails": member_emails,
        "admin_emails": admin_emails,
        "user_emails": user_emails,
        "invite_emails": invite_emails,
        "invite_role": invite_role,
        "admin_invite_emails": admin_invite_emails,
        "user_invite_emails": user_invite_emails,
        "pending_join_requests": [],
    }))
}
