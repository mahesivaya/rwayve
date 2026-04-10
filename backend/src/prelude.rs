// 📦 Logger functions (your custom ones)
pub use crate::logging::logger::{
    log_auth
};

// 🌐 Actix Web
pub use actix_web::{web, HttpResponse, HttpRequest, Responder, Error, get, post};

// 🔌 WebSocket (Actix actors)
pub use actix::{Actor, StreamHandler, ActorContext, AsyncContext};

// 🗄️ Database (SQLx)
pub use sqlx::{PgPool, FromRow, Row};       

// 📦 Serialization
pub use serde::{Serialize, Deserialize};
pub use serde_json::Value;

// ⏱️ Date & Time
pub use chrono::{NaiveDateTime, NaiveDate, NaiveTime};

// 🌍 HTTP client (Gmail API etc.)
pub use reqwest::Client;

// ⚡ Async utilities
pub use futures::stream::{FuturesUnordered, StreamExt};

// 🧠 Global state helpers
pub use once_cell::sync::Lazy;

// 📁 File handling
pub use std::fs;

// 🔐 Encoding
pub use base64::{engine::general_purpose::URL_SAFE_NO_PAD};

// ❗ Error handling
pub use anyhow::Result;

// 🚀 Constants
pub const MAX_EMAIL_CONCURRENCY: usize = 20;
pub const BATCH_SIZE: usize = 50;
