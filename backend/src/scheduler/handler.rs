use crate::models::scheduler::{CreateMeeting, Meeting};
use crate::prelude::*;
use actix_web::{HttpRequest, HttpResponse, delete, post, put, web};
use base64::Engine;
use chrono::Utc;
use serde_json::json;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{NaiveDate, NaiveTime};
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

    let decoded = crate::security::jwt::decode_jwt(token)
        .ok_or_else(|| HttpResponse::Unauthorized().body("Invalid token"))?;

    Ok(decoded.sub)
}

// ================= REQUEST STRUCT =================
pub struct MeetingEmailRequest {
    pub user_id: i32,
    pub participants: Vec<String>,
    pub title: String,
    pub date: NaiveDate,
    pub start: NaiveTime,
    pub end: NaiveTime,
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

    let body = format!(
        "📅 Meeting Invitation\n\nTitle: {}\nDate: {}\nStart: {}\nEnd: {}\n\n-- Wayve Scheduler",
        title, date, start_str, end_str
    );

    let to_list = valid_participants.join(",");

    let raw_message = format!(
        "From: {}\r\n\
To: {}\r\n\
Subject: Meeting: {}\r\n\
Content-Type: text/plain; charset=\"UTF-8\"\r\n\r\n{}",
        sender_email, to_list, title, body
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

    // 🔥 Prevent past meetings
    let now = Utc::now().naive_utc();
    if date == now.date() && start_time <= now.time() {
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
        SELECT id, title, date, start_time, end_time
        FROM meetings
        WHERE user_id = $1
        ORDER BY date, start_time
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
    path: web::Path<i32>,
    data: web::Json<CreateMeeting>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
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

    // ================= RESPONSE =================
    HttpResponse::Ok().json(json!({
        "message": "Meeting updated successfully"
    }))
}

#[delete("/meetings/{id}")]
pub async fn delete_meeting(path: web::Path<i32>, pool: web::Data<PgPool>) -> HttpResponse {
    let id = path.into_inner();

    let result = sqlx::query("DELETE FROM meetings WHERE id = $1")
        .bind(id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(json!({
            "message": "Meeting deleted"
        })),
        Err(e) => {
            println!("❌ Delete error FULL: {:#?}", e);
            HttpResponse::InternalServerError().body("Failed to delete meeting")
        }
    }
}
