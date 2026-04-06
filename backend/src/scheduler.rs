use crate::prelude::*;


// 🔥 MODEL
#[derive(Serialize, FromRow)]
pub struct Meeting {
    pub id: i32,
    pub title: String,
    pub date: NaiveDate,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
}

// 🔥 INPUT
#[derive(Deserialize)]
pub struct CreateMeeting {
    pub title: String,
    pub date: String,
    pub start: i32,
    pub end: i32,
}

// 🔥 HELPER
fn minutes_to_time(mins: i32) -> NaiveTime {
    let h = mins / 60;
    let m = mins % 60;
    NaiveTime::from_hms_opt(h as u32, m as u32, 0).unwrap()
}

// 🔥 CREATE
#[post("/meetings")]
pub async fn create_meeting(
    pool: web::Data<PgPool>,
    data: web::Json<CreateMeeting>,
) -> HttpResponse {

    let start_time = minutes_to_time(data.start);
    let end_time = minutes_to_time(data.end);
    let date = NaiveDate::parse_from_str(&data.date, "%Y-%m-%d").unwrap();

    let result = sqlx::query(
        r#"
        INSERT INTO meetings (title, date, start_time, end_time)
        VALUES ($1, $2, $3, $4)
        "#
    )
    .bind(&data.title)
    .bind(date)
    .bind(start_time)
    .bind(end_time)
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

// 🔥 GET
#[get("/meetings")]
pub async fn get_meetings(pool: web::Data<PgPool>) -> HttpResponse {

    let result = sqlx::query_as::<_, Meeting>(
        r#"
        SELECT id, title, date, start_time, end_time
        FROM meetings
        ORDER BY date, start_time
        "#
    )
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