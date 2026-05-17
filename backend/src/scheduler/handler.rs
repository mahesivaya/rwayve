use crate::models::scheduler::{CreateMeeting, Meeting};
use crate::prelude::*;
use crate::scheduler::auth::get_user_id;
use crate::scheduler::email_notifications::{
    MeetingEmailKind, MeetingEmailRequest, send_meeting_emails,
};
use crate::scheduler::time::minutes_to_time;
use crate::scheduler::zoom::create_zoom_meeting;
use actix_web::{HttpRequest, HttpResponse, delete, post, put, web};
use chrono::{TimeZone, Utc};
use chrono_tz::Tz;
use moka::future::Cache as MokaCache;
use serde_json::json;
use std::str::FromStr;
use std::time::Duration;
use tracing::{error, info, instrument, warn};

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use sqlx::{PgPool, Row};

const MEETINGS_CACHE_TTL_SECS: u64 = 60;
const MEETINGS_CACHE_MAX_CAPACITY: u64 = 10_000;

static MEETINGS_CACHE: Lazy<MokaCache<i32, Vec<Meeting>>> = Lazy::new(|| {
    MokaCache::builder()
        .max_capacity(MEETINGS_CACHE_MAX_CAPACITY)
        .time_to_live(Duration::from_secs(MEETINGS_CACHE_TTL_SECS))
        .build()
});

// ================= CREATE MEETING =================

#[derive(Serialize)]
struct MeetingResponse {
    message: String,
    meeting_id: i32,
}

#[post("/meetings")]
#[instrument(target = "scheduler", skip(req, pool, data), fields(title = %data.title))]
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

    // ================= ZOOM MEETING =================
    // Tolerant: if Zoom is misconfigured or fails, still create the meeting
    // without a join link rather than blocking the user.
    let duration_min = (end_time - start_time).num_minutes();
    let zoom_join_url = match create_zoom_meeting(&data.title, meeting_utc, duration_min).await {
        Ok(url) => Some(url),
        Err(e) => {
            warn!(
                "Zoom meeting create failed: {} (continuing without join url)",
                e
            );
            None
        }
    };

    // ================= TRANSACTION =================
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            error!(target: "db", error = ?e, "create_meeting tx begin failed");
            return HttpResponse::InternalServerError().finish();
        }
    };

    // ================= INSERT MEETING =================
    let meeting = match sqlx::query(
        r#"
        INSERT INTO meetings (title, date, start_time, end_time, user_id, zoom_join_url)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
    )
    .bind(&data.title)
    .bind(date)
    .bind(start_time)
    .bind(end_time)
    .bind(user_id)
    .bind(&zoom_join_url)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(m) => m,
        Err(e) => {
            error!(target: "db", user_id, error = ?e, "meeting insert failed");
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
        error!(target: "db", meeting_id, error = ?e, "participants insert failed");
        let _ = tx.rollback().await;
        return HttpResponse::InternalServerError().finish();
    }

    // ================= COMMIT =================
    if let Err(e) = tx.commit().await {
        error!(target: "db", meeting_id, error = ?e, "create_meeting tx commit failed");
        return HttpResponse::InternalServerError().finish();
    }
    MEETINGS_CACHE.invalidate(&user_id).await;

    info!(
        "Meeting created: id={} user_id={} title=\"{}\"",
        meeting_id, user_id, data.title
    );

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
        zoom_join_url: zoom_join_url.clone(),
    };

    actix_web::rt::spawn(async move {
        if let Err(e) = send_meeting_emails(pool_clone.get_ref(), email_req).await {
            warn!(target: "scheduler", meeting_id, error = %e, "invite email failed");
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
#[instrument(target = "http", skip(req, pool))]
pub async fn get_meetings(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    if let Some(cached) = MEETINGS_CACHE.get(&user_id).await {
        return HttpResponse::Ok().json(cached);
    }

    let result = sqlx::query_as::<_, Meeting>(
        r#"
        SELECT
            m.id,
            m.title,
            m.date,
            m.start_time,
            m.end_time,
            m.zoom_join_url,
            m.source,
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
        Ok(rows) => {
            MEETINGS_CACHE.insert(user_id, rows.clone()).await;
            HttpResponse::Ok().json(rows)
        }
        Err(e) => {
            error!(target: "db", user_id, error = ?e, "get_meetings failed");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[put("/meetings/{id}")]
#[instrument(target = "scheduler", skip(req, path, data, pool))]
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
        "SELECT title, date, start_time, end_time, zoom_join_url FROM meetings WHERE id = $1",
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
            row.get::<Option<String>, _>("zoom_join_url"),
        )),
        Ok(None) => None,
        Err(e) => {
            error!(target: "db", meeting_id = id, error = ?e, "update_meeting load failed");
            return HttpResponse::InternalServerError().finish();
        }
    };

    // ================= TRANSACTION =================
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            error!(target: "db", meeting_id = id, error = ?e, "update_meeting tx begin failed");
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
        error!(target: "db", meeting_id = id, error = ?e, "meeting update failed");
        let _ = tx.rollback().await;
        return HttpResponse::InternalServerError().body("Failed to update meeting");
    }

    // ================= DELETE OLD PARTICIPANTS =================
    if let Err(e) = sqlx::query("DELETE FROM meeting_participants WHERE meeting_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await
    {
        error!(target: "db", meeting_id = id, error = ?e, "delete old participants failed");
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
        error!(target: "db", meeting_id = id, error = ?e, "insert new participants failed");
        let _ = tx.rollback().await;
        return HttpResponse::InternalServerError().finish();
    }

    // ================= COMMIT =================
    if let Err(e) = tx.commit().await {
        error!(target: "db", meeting_id = id, error = ?e, "update_meeting tx commit failed");
        return HttpResponse::InternalServerError().finish();
    }
    MEETINGS_CACHE.invalidate(&user_id).await;

    info!("Meeting updated: id={} user_id={}", id, user_id);

    // ================= NOTIFY ON CONTENT CHANGES =================
    // Email participants only when title/date/start/end actually changed —
    // a participant-list-only edit should not spam everyone.
    let (content_changed, existing_zoom_url) = match &existing {
        Some((t, d, s, e, z)) => (
            t != &data.title || d != &date || s != &start_time || e != &end_time,
            z.clone(),
        ),
        None => (false, None),
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
                zoom_join_url: existing_zoom_url,
            };
            actix_web::rt::spawn(async move {
                if let Err(e) = send_meeting_emails(pool_clone.get_ref(), email_req).await {
                    warn!(target: "scheduler", meeting_id = id, error = %e, "update email failed");
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
#[instrument(target = "scheduler", skip(req, path, pool))]
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
        "SELECT title, date, start_time, end_time, zoom_join_url FROM meetings WHERE id = $1",
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
            row.get::<Option<String>, _>("zoom_join_url"),
        )),
        Ok(None) => None,
        Err(e) => {
            error!(target: "db", meeting_id = id, error = ?e, "delete_meeting snapshot load failed");
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
        Ok(rows) => rows
            .into_iter()
            .map(|r| r.get::<String, _>("email"))
            .collect(),
        Err(e) => {
            warn!(target: "db", meeting_id = id, error = ?e, "delete_meeting load participants failed");
            Vec::new()
        }
    };

    let result = sqlx::query("DELETE FROM meetings WHERE id = $1")
        .bind(id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => {
            MEETINGS_CACHE.invalidate(&user_id).await;
            if let Some((title, date, start_time, end_time, zoom_join_url)) = snapshot
                && !participants.is_empty()
            {
                let pool_clone = pool.clone();
                let email_req = MeetingEmailRequest {
                    user_id,
                    participants,
                    title,
                    date,
                    start: start_time,
                    end: end_time,
                    kind: MeetingEmailKind::Cancel,
                    zoom_join_url,
                };
                actix_web::rt::spawn(async move {
                    if let Err(e) = send_meeting_emails(pool_clone.get_ref(), email_req).await {
                        warn!(target: "scheduler", meeting_id = id, error = %e, "cancel email failed");
                    }
                });
            }
            info!("Meeting deleted: id={} user_id={}", id, user_id);
            HttpResponse::Ok().json(json!({
                "message": "Meeting deleted"
            }))
        }
        Err(e) => {
            error!("Meeting delete failed (id={}): {:?}", id, e);
            HttpResponse::InternalServerError().body("Failed to delete meeting")
        }
    }
}
