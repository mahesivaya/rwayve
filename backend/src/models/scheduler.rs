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
    pub participants: Vec<String>,
}