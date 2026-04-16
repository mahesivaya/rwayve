use crate::prelude::*;

// ================= MODEL =================
#[derive(Serialize, FromRow)]
pub struct Meeting {
    pub id: i32,
    pub title: String,
    pub date: NaiveDate,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
}

// ================= INPUT =================
#[derive(Deserialize)]
pub struct CreateMeeting {
    pub title: String,
    pub date: String,
    pub start: i32,
    pub end: i32,
}

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
        Ok(_) => HttpResponse::Ok().body("Created"),
        Err(e) => {
            println!("DB error: {:?}", e);
            HttpResponse::InternalServerError().finish()
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