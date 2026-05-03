use crate::models::scheduler::{CreateMeeting, Meeting};
use crate::prelude::*;
use actix_web::{HttpRequest, HttpResponse, delete, post, put, web};
use base64::Engine;
use chrono::{TimeZone, Utc};
use chrono_tz::Tz;
use serde_json::json;
use std::str::FromStr;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use sqlx::{PgPool, Row};

// ================= HELPER =================
fn minutes_to_time(mins: i32) -> NaiveTime {
    let h = mins / 60;
    let m = mins % 60;
    NaiveTime::from_hms_opt(h as u32, m as u32, 0)
        .unwrap_or_else(|| NaiveTime::from_hms_opt(0, 0, 0).unwrap())
}

// ================= EXTRACT USER =================
fn get_user_id(req: &HttpRequest) -> Result<i32, HttpResponse> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or_else(|| HttpResponse::Unauthorized().body("Missing token"))?;

    let decoded = crate::security::jwt::decode_jwt(&token)
        .ok_or_else(|| HttpResponse::Unauthorized().body("Invalid token"))?;

    Ok(decoded.sub)
}

// ================= REQUEST STRUCT =================
#[derive(Clone, Copy)]
pub enum MeetingEmailKind {
    Invite,
    Update,
    Cancel,
}

pub struct MeetingEmailRequest {
    pub user_id: i32,
    pub participants: Vec<String>,
    pub title: String,
    pub date: NaiveDate,
    pub start: NaiveTime,
    pub end: NaiveTime,
    pub kind: MeetingEmailKind,
}

// ================= MAIN FUNCTION =================
pub async fn send_meeting_emails(pool: &PgPool, req: MeetingEmailRequest) -> Result<(), String> {
    let MeetingEmailRequest {
        user_id,
        participants,
        title,
        date,
        start,
        end,
        kind,
    } = req;

    let row = sqlx::query(
        "SELECT access_token, email FROM email_accounts 
         WHERE user_id = $1 AND is_active = true LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("DB error: {:?}", e))?;

    let row = match row {
        Some(r) => r,
        None => return Err("No active Gmail account found".into()),
    };

    let access_token: String = row.get("access_token");
    let sender_email: String = row.get("email");

    if access_token.is_empty() {
        return Err("Missing access token".into());
    }

    let valid_participants: Vec<String> = participants
        .into_iter()
        .map(|e| e.trim().to_lowercase())
        .filter(|e| e.contains("@") && e.contains("."))
        .collect();

    if valid_participants.is_empty() {
        return Err("No valid participants".into());
    }

    let start_str = start.format("%H:%M").to_string();
    let end_str = end.format("%H:%M").to_string();

    let (header, subject_prefix) = match kind {
        MeetingEmailKind::Invite => ("📅 Meeting Invitation", "Meeting"),
        MeetingEmailKind::Update => ("✏️ Meeting Updated", "Updated"),
        MeetingEmailKind::Cancel => ("❌ Meeting Cancelled", "Cancelled"),
    };

    let body = format!(
        "{}\n\nTitle: {}\nDate: {}\nStart: {}\nEnd: {}\n\n-- Wayve Scheduler",
        header, title, date, start_str, end_str
    );

    let to_list = valid_participants.join(",");

    let raw_message = format!(
        "From: {}\r\n\
To: {}\r\n\
Subject: {}: {}\r\n\
Content-Type: text/plain; charset=\"UTF-8\"\r\n\r\n{}",
        sender_email, to_list, subject_prefix, title, body
    );

    let encoded = URL_SAFE_NO_PAD.encode(raw_message);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("HTTP client error: {:?}", e))?;

    let res = client
        .post("https://gmail.googleapis.com/gmail/v1/users/me/messages/send")
        .bearer_auth(access_token)
        .json(&json!({ "raw": encoded }))
        .send()
        .await
        .map_err(|e| format!("HTTP send error: {:?}", e))?;

    if !res.status().is_success() {
        let text = res.text().await.unwrap_or_default();
        return Err(format!("Gmail failed: {}", text));
    }

    println!("✅ Meeting emails sent successfully");

    Ok(())
}

// ================= CREATE MEETING =================

#[derive(Serialize)]
struct MeetingResponse {
    message: String,
    meeting_id: i32,
}

#[post("/meetings")]
pub async fn create_meeting(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    data: web::Json<CreateMeeting>,
) -> impl Responder {
    // ================= AUTH =================
    let user_id = match get_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    // ================= VALIDATION =================
    if data.title.trim().is_empty() {
        return HttpResponse::BadRequest().body("Title is required");
    }

    if data.participants.is_empty() {
        return HttpResponse::BadRequest().body("At least one participant required");
    }

    // ================= CLEAN PARTICIPANTS =================
    let participants: Vec<String> = data
        .participants
        .iter()
        .map(|e| e.trim().to_lowercase())
        .filter(|e| e.contains("@") && e.contains("."))
        .collect();

    if participants.is_empty() {
        return HttpResponse::BadRequest().body("Invalid participant emails");
    }

    // ================= TIME =================
    let start_time: NaiveTime = minutes_to_time(data.start);
    let end_time: NaiveTime = minutes_to_time(data.end);

    if start_time >= end_time {
        return HttpResponse::BadRequest().body("Invalid time range");
    }

    // ================= DATE =================
    let date = match NaiveDate::parse_from_str(&data.date, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => return HttpResponse::BadRequest().body("Invalid date"),
    };

    // Prevent past meetings — interpret date+start as wall-clock time in the
    // client's IANA timezone. Default to UTC if the client didn't send one
    // (or sent something unparseable) — UTC is a neutral fallback that won't
    // spuriously reject future meetings the way a hardcoded NY zone did.
    let tz: Tz = data
        .tz
        .as_deref()
        .and_then(|s| Tz::from_str(s).ok())
        .unwrap_or(Tz::UTC);

    let naive = NaiveDateTime::new(date, start_time);
    let meeting_utc = match tz.from_local_datetime(&naive).single() {
        Some(dt) => dt.with_timezone(&Utc),
        None => return HttpResponse::BadRequest().body("Invalid date/time"),
    };
    if meeting_utc <= Utc::now() {
        return HttpResponse::BadRequest().body("Meeting cannot be in the past");
    }

    // ================= TRANSACTION =================
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            println!("❌ TX error: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // ================= INSERT MEETING =================
    let meeting = match sqlx::query(
        r#"
        INSERT INTO meetings (title, date, start_time, end_time, user_id)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(&data.title)
    .bind(date)
    .bind(start_time)
    .bind(end_time)
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(m) => m,
        Err(e) => {
            println!("❌ Meeting insert error: {:?}", e);
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().finish();
        }
    };

    let meeting_id: i32 = meeting.get("id");

    // ================= INSERT PARTICIPANTS =================
    let insert_participants = sqlx::query(
        r#"
        INSERT INTO meeting_participants (meeting_id, email, user_id)
        SELECT 
            $1,
            v.email,
            u.id
        FROM UNNEST($2::text[]) AS v(email)
        LEFT JOIN users u 
        ON LOWER(TRIM(u.email)) = LOWER(TRIM(v.email))
        ON CONFLICT DO NOTHING;
        "#,
    )
    .bind(meeting_id)
    .bind(&participants)
    .execute(&mut *tx)
    .await;

    if let Err(e) = insert_participants {
        println!("❌ Participants insert error: {:?}", e);
        let _ = tx.rollback().await;
        return HttpResponse::InternalServerError().finish();
    }

    // ================= COMMIT =================
    if let Err(e) = tx.commit().await {
        println!("❌ TX commit error: {:?}", e);
        return HttpResponse::InternalServerError().finish();
    }

    // ================= BACKGROUND EMAIL =================
    let pool_clone = pool.clone();

    let email_req = MeetingEmailRequest {
        user_id,
        participants: participants.clone(),
        title: data.title.clone(),
        date,
        start: start_time,
        end: end_time,
        kind: MeetingEmailKind::Invite,
    };

    actix_web::rt::spawn(async move {
        if let Err(e) = send_meeting_emails(pool_clone.get_ref(), email_req).await {
            println!("❌ Email sending failed: {:?}", e);
        }
    });

    // ================= RESPONSE =================
    HttpResponse::Ok().json(MeetingResponse {
        message: "Meeting created successfully".into(),
        meeting_id,
    })
}

// ================= GET =================
#[get("/meetings")]
pub async fn get_meetings(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let result = sqlx::query_as::<_, Meeting>(
        r#"
        SELECT
            m.id,
            m.title,
            m.date,
            m.start_time,
            m.end_time,
            COALESCE(
                ARRAY_AGG(mp.email) FILTER (WHERE mp.email IS NOT NULL),
                ARRAY[]::text[]
            ) AS participants
        FROM meetings m
        LEFT JOIN meeting_participants mp ON mp.meeting_id = m.id
        WHERE m.user_id = $1
        GROUP BY m.id
        ORDER BY m.date, m.start_time
        "#,
    )
    .bind(user_id)
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(rows) => HttpResponse::Ok().json(rows),
        Err(e) => {
            println!("DB error: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[put("/meetings/{id}")]
pub async fn update_meeting(
    req: HttpRequest,
    path: web::Path<i32>,
    data: web::Json<CreateMeeting>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let id = path.into_inner();

    // ================= VALIDATION =================
    if data.title.trim().is_empty() {
        return HttpResponse::BadRequest().body("Title is required");
    }

    if data.participants.is_empty() {
        return HttpResponse::BadRequest().body("At least one participant required");
    }

    let date = match chrono::NaiveDate::parse_from_str(&data.date, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => return HttpResponse::BadRequest().body("Invalid date format"),
    };

    let start_time = minutes_to_time(data.start);
    let end_time = minutes_to_time(data.end);

    if start_time >= end_time {
        return HttpResponse::BadRequest().body("Invalid time range");
    }

    // ================= LOAD EXISTING (for change detection) =================
    let existing = match sqlx::query(
        "SELECT title, date, start_time, end_time FROM meetings WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(row)) => Some((
            row.get::<String, _>("title"),
            row.get::<NaiveDate, _>("date"),
            row.get::<NaiveTime, _>("start_time"),
            row.get::<NaiveTime, _>("end_time"),
        )),
        Ok(None) => None,
        Err(e) => {
            println!("❌ Load existing meeting error: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // ================= TRANSACTION =================
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            println!("❌ TX error: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // ================= UPDATE MEETING =================
    let update = sqlx::query(
        r#"
        UPDATE meetings
        SET title=$1, date=$2, start_time=$3, end_time=$4
        WHERE id=$5
        "#,
    )
    .bind(&data.title)
    .bind(date)
    .bind(start_time)
    .bind(end_time)
    .bind(id)
    .execute(&mut *tx)
    .await;

    if let Err(e) = update {
        println!("❌ Update error: {:?}", e);
        let _ = tx.rollback().await;
        return HttpResponse::InternalServerError().body("Failed to update meeting");
    }

    // ================= DELETE OLD PARTICIPANTS =================
    if let Err(e) = sqlx::query("DELETE FROM meeting_participants WHERE meeting_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await
    {
        println!("❌ Delete participants error: {:?}", e);
        let _ = tx.rollback().await;
        return HttpResponse::InternalServerError().finish();
    }

    // ================= INSERT NEW PARTICIPANTS =================
    let insert = sqlx::query(
        r#"
        INSERT INTO meeting_participants (meeting_id, email, user_id)
        SELECT
            $1,
            v.email,
            u.id
        FROM UNNEST($2::text[]) AS v(email)
        LEFT JOIN users u
        ON LOWER(TRIM(u.email)) = LOWER(TRIM(v.email))
        "#,
    )
    .bind(id)
    .bind(&data.participants)
    .execute(&mut *tx)
    .await;

    if let Err(e) = insert {
        println!("❌ Insert participants error: {:?}", e);
        let _ = tx.rollback().await;
        return HttpResponse::InternalServerError().finish();
    }

    // ================= COMMIT =================
    if let Err(e) = tx.commit().await {
        println!("❌ TX commit error: {:?}", e);
        return HttpResponse::InternalServerError().finish();
    }

    // ================= NOTIFY ON CONTENT CHANGES =================
    // Email participants only when title/date/start/end actually changed —
    // a participant-list-only edit should not spam everyone.
    let content_changed = match &existing {
        Some((t, d, s, e)) => {
            t != &data.title || d != &date || s != &start_time || e != &end_time
        }
        None => false,
    };

    if content_changed {
        let participants: Vec<String> = data
            .participants
            .iter()
            .map(|e| e.trim().to_lowercase())
            .filter(|e| e.contains("@") && e.contains("."))
            .collect();

        if !participants.is_empty() {
            let pool_clone = pool.clone();
            let email_req = MeetingEmailRequest {
                user_id,
                participants,
                title: data.title.clone(),
                date,
                start: start_time,
                end: end_time,
                kind: MeetingEmailKind::Update,
            };
            actix_web::rt::spawn(async move {
                if let Err(e) = send_meeting_emails(pool_clone.get_ref(), email_req).await {
                    println!("❌ Update email failed: {:?}", e);
                }
            });
        }
    }

    // ================= RESPONSE =================
    HttpResponse::Ok().json(json!({
        "message": "Meeting updated successfully"
    }))
}

#[delete("/meetings/{id}")]
pub async fn delete_meeting(
    req: HttpRequest,
    path: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let id = path.into_inner();

    // Snapshot meeting + participants before deletion so we can email them.
    let meeting_row = sqlx::query(
        "SELECT title, date, start_time, end_time FROM meetings WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await;

    let snapshot = match meeting_row {
        Ok(Some(row)) => Some((
            row.get::<String, _>("title"),
            row.get::<NaiveDate, _>("date"),
            row.get::<NaiveTime, _>("start_time"),
            row.get::<NaiveTime, _>("end_time"),
        )),
        Ok(None) => None,
        Err(e) => {
            println!("❌ Load meeting (delete) error: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let participants: Vec<String> = match sqlx::query(
        "SELECT email FROM meeting_participants WHERE meeting_id = $1",
    )
    .bind(id)
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(rows) => rows.into_iter().map(|r| r.get::<String, _>("email")).collect(),
        Err(e) => {
            println!("❌ Load participants (delete) error: {:?}", e);
            Vec::new()
        }
    };

    let result = sqlx::query("DELETE FROM meetings WHERE id = $1")
        .bind(id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => {
            if let Some((title, date, start_time, end_time)) = snapshot {
                if !participants.is_empty() {
                    let pool_clone = pool.clone();
                    let email_req = MeetingEmailRequest {
                        user_id,
                        participants,
                        title,
                        date,
                        start: start_time,
                        end: end_time,
                        kind: MeetingEmailKind::Cancel,
                    };
                    actix_web::rt::spawn(async move {
                        if let Err(e) = send_meeting_emails(pool_clone.get_ref(), email_req).await {
                            println!("❌ Cancel email failed: {:?}", e);
                        }
                    });
                }
            }
            HttpResponse::Ok().json(json!({
                "message": "Meeting deleted"
            }))
        }
        Err(e) => {
            println!("❌ Delete error FULL: {:#?}", e);
            HttpResponse::InternalServerError().body("Failed to delete meeting")
        }
    }
}
