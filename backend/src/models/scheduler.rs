use serde::{Serialize, Deserialize};
use sqlx::FromRow;

#[derive(Serialize, FromRow)]
struct Meeting {
    id: i32,
    title: String,
    date: chrono::NaiveDate,
    start_time: chrono::NaiveTime,
    end_time: chrono::NaiveTime,
}

#[derive(Deserialize)]
struct CreateMeeting {
    title: String,
    date: String,
    start: i32, // minutes
    end: i32,
}