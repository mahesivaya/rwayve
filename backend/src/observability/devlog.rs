//! Tiny zero-dep file logger used while `tracing-subscriber` is disabled.
//!
//! Writes lines like `2026-05-07 10:31:22.481 INFO  User registered: demo@gmail.com`
//! to `backend/logs/dev.log` (append, daily file kept simple — single rolling
//! file). Also mirrors to stderr so dev console shows the same text.
//!
//! Usage:
//! ```ignore
//! use crate::{dev_info, dev_warn, dev_error};
//! dev_info!("Server started on :{}", port);
//! ```

use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;
use std::sync::Mutex;

use chrono::Local;
use once_cell::sync::OnceCell;

static FILE: OnceCell<Mutex<std::fs::File>> = OnceCell::new();

/// Open `logs/dev.log` for append. Safe to call once on startup; subsequent
/// calls are no-ops.
pub fn init_devlog() {
    if FILE.get().is_some() {
        return;
    }
    let _ = create_dir_all("logs");
    match OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs/dev.log")
    {
        Ok(f) => {
            let _ = FILE.set(Mutex::new(f));
        }
        Err(e) => {
            eprintln!("devlog: failed to open logs/dev.log: {e}");
        }
    }
}

/// Implementation backing the `dev_*!` macros. Public because the macros
/// expand to a call here, but prefer the macros for caller ergonomics.
pub fn write(level: &str, msg: &str) {
    let line = format!(
        "{} {:5} {}\n",
        Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
        level,
        msg
    );
    // Mirror to stderr so devs see logs without tailing the file.
    eprint!("{line}");
    if let Some(m) = FILE.get()
        && let Ok(mut f) = m.lock()
    {
        let _ = f.write_all(line.as_bytes());
    }
}

#[macro_export]
macro_rules! dev_info {
    ($($arg:tt)*) => {
        $crate::observability::devlog::write("INFO", &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! dev_warn {
    ($($arg:tt)*) => {
        $crate::observability::devlog::write("WARN", &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! dev_error {
    ($($arg:tt)*) => {
        $crate::observability::devlog::write("ERROR", &format!($($arg)*))
    };
}
