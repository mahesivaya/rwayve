use crate::prelude::*;
use base64::Engine;
use actix_web::{put, delete};
use serde_json::json;


use crate::models::scheduler::{Meeting,CreateMeeting};

// ================= HELPER =================
fn minutes_to_time(mins: i32) -> NaiveTime {
    let h = mins / 60;
    let m = mins % 60;
    NaiveTime::from_hms_opt(h as u32, m as u32, 0).unwrap()
}

// ================= EXTRACT USER =================
fn get_user_id(req: &HttpRequest) -> Result<i32, HttpResponse> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or_else(|| HttpResponse::Unauthorized().body("Missing token"))?;

    let decoded = crate::models::auth::decode_jwt(token)
        .ok_or_else(|| HttpResponse::Unauthorized().body("Invalid token"))?;

    Ok(decoded.sub)
}


use sqlx::{PgPool, Row};
use chrono::{NaiveDate, NaiveTime};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

pub async fn send_meeting_emails(
    pool: &PgPool,
    user_id: i32,
    participants: Vec<String>,
    title: String,
    date: NaiveDate,
    start: NaiveTime,
    end: NaiveTime,
) {
    // ✅ Get user's active Gmail account
    let result = sqlx::query(
        "SELECT access_token, email FROM email_accounts 
         WHERE user_id = $1 AND is_active = true LIMIT 1"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await;

    let row: sqlx::postgres::PgRow = match result {
        Ok(Some(r)) => r,
        Ok(None) => {
            println!("❌ No active Gmail account found");
            return;
        }
        Err(e) => {
            println!("❌ DB error fetching email account: {:?}", e);
            return;
        }
    };

    let access_token: String = row.get("access_token");
    let sender_email: String = row.get("email");

    if access_token.is_empty() {
        println!("❌ Missing access token");
        return;
    }

    // ✅ Clean participants
    let valid_participants: Vec<String> = participants
        .into_iter()
        .map(|e| e.trim().to_lowercase())   // ✅ FIX
        .filter(|e| e.contains("@") && e.contains("."))
        .collect();

    if valid_participants.is_empty() {
        println!("⚠️ No valid participants to send");
        return;
    }

    // ✅ Convert times to readable format
    let start_str = start.format("%H:%M").to_string();
    let end_str = end.format("%H:%M").to_string();

    // ✅ Email body
    let body = format!(
"📅 Meeting Invitation

Title: {}
Date: {}
Start: {}
End: {}

You have been invited to a meeting.

-- Wayve Scheduler",
        title, date, start_str, end_str
    );

    // ✅ Send ONE email to all participants
    let to_list = valid_participants.join(",");

    let raw_message = format!(
        "From: {}\r\n\
        To: {}\r\n\
        Subject: Meeting: {}\r\n\
        Content-Type: text/plain; charset=\"UTF-8\"\r\n\
        \r\n\
        {}",
        sender_email,
        to_list,
        title,
        body
        );

    // ✅ Encode for Gmail
    let encoded = URL_SAFE_NO_PAD.encode(raw_message);

    // ✅ Send email
    let client = reqwest::Client::builder()
    .timeout(std::time::Duration::from_secs(10))
    .build()
    .unwrap();

    let res = client
        .post("https://gmail.googleapis.com/gmail/v1/users/me/messages/send")
        .bearer_auth(access_token)
        .json(&serde_json::json!({
            "raw": encoded
        }))
        .send()
        .await;

    match res {
        Ok(r) => {
            if !r.status().is_success() {
                let text = r.text().await.unwrap_or_default();
                println!(
                    "❌ Gmail send failed | user={} | title={} | error={}",
                    user_id, title, text
                );
            } else {
                println!("✅ Meeting emails sent successfully");
            }
        }
        Err(e) => {
            println!("❌ HTTP error sending email: {:?}", e);
        }
    }
}

// ================= CREATE =================
#[post("/meetings")]
pub async fn create_meeting(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    data: web::Json<CreateMeeting>,
) -> HttpResponse {

    let user_id = match get_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let start_time = minutes_to_time(data.start);
    let end_time = minutes_to_time(data.end);

    let date = match NaiveDate::parse_from_str(&data.date, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => return HttpResponse::BadRequest().body("Invalid date"),
    };

    let result = sqlx::query(
        r#"
        INSERT INTO meetings (title, date, start_time, end_time, user_id)
        VALUES ($1, $2, $3, $4, $5)
        "#
    )
    .bind(&data.title)
    .bind(date)
    .bind(start_time)
    .bind(end_time)
    .bind(user_id)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => {

            // ✅ Clone everything BEFORE spawn
            let pool_clone = pool.clone();
            let participants = data.participants.clone();
            let title = data.title.clone();
            let date_clone = date;
            let start_clone = start_time;
            let end_clone = end_time;
            let user_id_clone = user_id;

            // ✅ Background email sending

actix_web::rt::spawn(async move {
    send_meeting_emails(
        pool_clone.get_ref(),
        user_id_clone,
        participants,
        title,
        date_clone,
        start_clone,
        end_clone,
    )
    .await;
});

            // ✅ Immediate response
            HttpResponse::Ok().json(serde_json::json!({
                "message": "Meeting created. Emails sending in background"
            }))
        }

        Err(e) => {
            println!("DB error: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to create meeting")
        }
    }
}




// ================= GET =================
#[get("/meetings")]
pub async fn get_meetings(
    req: HttpRequest,
    pool: web::Data<PgPool>,
) -> HttpResponse {

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
        "#
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


fn format_meeting_email(title: &str, date: &NaiveDate, start: NaiveTime, end: NaiveTime) -> String {
    format!(
        "Subject: Meeting Scheduled\r\n\
         Content-Type: text/plain; charset=\"UTF-8\"\r\n\r\n\
         Your meeting has been scheduled.\n\n\
         Title: {}\n\
         Date: {}\n\
         Time: {} - {}\n",
        title,
        date,
        start.format("%H:%M"),
        end.format("%H:%M"),
    )
}

pub async fn send_email_direct(
    access_token: &str,
    _to: &str,
    raw_message: String,
) -> Result<(), reqwest::Error> {
    let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw_message);

    let client = reqwest::Client::new();

    client
        .post("https://gmail.googleapis.com/gmail/v1/users/me/messages/send")
        .bearer_auth(access_token)
        .json(&serde_json::json!({ "raw": encoded }))
        .send()
        .await?;

    Ok(())
}

#[put("/meetings/{id}")]
async fn update_meeting(
    path: web::Path<i32>,
    data: web::Json<CreateMeeting>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    let id = path.into_inner();

    let date = match chrono::NaiveDate::parse_from_str(&data.date, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => {
            return HttpResponse::BadRequest().body("Invalid date format");
        }
    };

    let result = sqlx::query(
        "UPDATE meetings 
         SET title=$1, date=$2, start_time=$3, end_time=$4 
         WHERE id=$5"
    )
    .bind(&data.title)
    .bind(date)
    .bind(minutes_to_time(data.start))
    .bind(minutes_to_time(data.end))
    .bind(id)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => {
            HttpResponse::Ok().json(json!({
                "message": "Meeting updated"
            }))
        }
        Err(e) => {
            println!("❌ Update error FULL: {:#?}", e);
            HttpResponse::InternalServerError().body("Failed to update meeting")
        }
    }
}


#[delete("/meetings/{id}")]
async fn delete_meeting(
    path: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    let id = path.into_inner();

    let result = sqlx::query("DELETE FROM meetings WHERE id = $1")
        .bind(id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => {
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