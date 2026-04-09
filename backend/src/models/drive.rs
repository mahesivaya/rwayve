use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use chrono::NaiveDateTime;

//
// 🔹 1. DB MODEL (matches database)
//
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DriveFile {
    pub id: i64,
    pub user_id: i64,

    pub name: String,
    pub file_type: Option<String>,

    pub drive_file_id: Option<String>, // 🔥 Google file ID
    pub drive_url: Option<String>,     // 🔥 Google link

    pub file_path: Option<String>,     // local path (if any)
    pub size: Option<i64>,

    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,

    pub is_deleted: bool,
}

//
// 🔹 2. CREATE REQUEST (API input)
//
#[derive(Debug, Deserialize)]
pub struct CreateDriveFile {
    pub user_id: i64,
    pub name: String,

    pub file_type: Option<String>,
    pub file_path: Option<String>,

    pub size: Option<i64>,
}

//
// 🔹 3. GOOGLE DRIVE RESPONSE (PARSED)
//
#[derive(Debug, Deserialize)]
pub struct GoogleDriveFile {
    pub id: String,
    pub name: String,
    pub mimeType: Option<String>,
    pub webViewLink: Option<String>,
    pub size: Option<String>,
    pub createdTime: Option<String>,
}

//
// 🔹 4. API RESPONSE (FRONTEND)
//
#[derive(Debug, Serialize)]
pub struct DriveFileResponse {
    pub id: i64,
    pub name: String,
    pub file_type: Option<String>,

    pub drive_url: Option<String>,
    pub size: Option<i64>,

    pub created_at: NaiveDateTime,
}