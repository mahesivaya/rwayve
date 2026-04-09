use fern::Dispatch;
use log::LevelFilter;
use chrono::Local;
use log::{info, error, warn};


pub fn init_logger() {
    std::fs::create_dir_all("logs").unwrap();
    let base_config = Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.target(),
                record.level(),
                message
            ))
        });

    // 🟢 App logs
    let app_log = fern::log_file("logs/app.log").unwrap();

    // 🔴 Error logs
    let error_log = fern::log_file("logs/error.log").unwrap();

    // 🔐 Auth logs
    let auth_log = fern::log_file("logs/auth.log").unwrap();

    // 📧 Email logs
    let email_log = fern::log_file("logs/email.log").unwrap();

    base_config
        .level(LevelFilter::Info)

        // Default (all logs → app.log)
        .chain(app_log)

        // Only errors → error.log
        .chain(
            Dispatch::new()
                .level(LevelFilter::Error)
                .chain(error_log)
        )

        // Auth logs (by target)
        .chain(
            Dispatch::new()
                .filter(|meta| meta.target().starts_with("auth"))
                .chain(auth_log)
        )

        // Email logs
        .chain(
            Dispatch::new()
                .filter(|meta| meta.target().starts_with("email"))
                .chain(email_log)
        )

        .apply()
        .unwrap();
}


pub fn log_auth(msg: impl AsRef<str>) {
    log::info!(target: "auth", "{}", msg.as_ref());
}

pub fn log_email(msg: &str) {
    info!(target: "email", "{}", msg);
}

pub fn log_error(msg: &str) {
    error!(target: "error", "{}", msg);
}

pub fn log_app(msg: &str) {
    info!(target: "app", "{}", msg);
}