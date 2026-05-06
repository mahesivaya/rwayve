//! Tracing setup for the whole binary.
//!
//! Routing:
//! - `app.log`    — every event at or above the configured level (default INFO)
//! - `error.log`  — every event at ERROR (for alerting / oncall paging)
//! - `auth.log`   — events with `target = "auth"` or whose target starts with
//!   `auth` / `wayve::auth`. Login, register, JWT decode failures.
//! - `email.log`  — events with `target = "email"`. Sync, body_worker, OAuth,
//!   calendar import, message endpoints.
//! - stdout       — same content as `app.log`, ANSI-coloured for local dev.
//!
//! Filtering: `RUST_LOG` env var controls verbosity, e.g.
//!   RUST_LOG=info,sqlx=warn,hyper=warn,wayve::email=debug
//!
//! `log::*` calls (from old code or third-party deps) are bridged into the
//! same tracing subscriber via `tracing-log::LogTracer`, so there's exactly
//! one event pipeline.

use std::io;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{Layer, registry};

/// Holds the WorkerGuards returned by tracing-appender's non-blocking writers.
/// The guards must outlive `main`, otherwise buffered logs are dropped at
/// shutdown. Stash them in a static so they live for the entire process.
static GUARDS: once_cell::sync::OnceCell<Vec<WorkerGuard>> = once_cell::sync::OnceCell::new();

pub fn init_logger() {
    std::fs::create_dir_all("logs").expect("create logs/ dir");

    // Bridge any remaining `log::*` calls (and third-party crates that use
    // `log`) into our tracing subscriber. Must be installed before the
    // subscriber is set as global default.
    let _ = tracing_log::LogTracer::init();

    // Default to INFO; overridable via RUST_LOG.
    let env_filter = || {
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info,sqlx=warn,hyper=warn,h2=warn,reqwest=warn"))
    };

    // Non-blocking file appenders. `daily` rolls at midnight and keeps each
    // day's logs separate — easier to ship to S3/Loki than a single ever-
    // growing file.
    let (app_w, app_g) = tracing_appender::non_blocking(rolling::daily("logs", "app.log"));
    let (err_w, err_g) = tracing_appender::non_blocking(rolling::daily("logs", "error.log"));
    let (auth_w, auth_g) = tracing_appender::non_blocking(rolling::daily("logs", "auth.log"));
    let (email_w, email_g) = tracing_appender::non_blocking(rolling::daily("logs", "email.log"));

    // Stdout layer — coloured, human-friendly, for `cargo run`.
    let stdout_layer = fmt::layer()
        .with_target(true)
        .with_writer(io::stdout)
        .with_ansi(true)
        .with_filter(env_filter());

    // Default app log — every level honoured by env_filter goes here.
    let app_layer = fmt::layer()
        .with_target(true)
        .with_writer(app_w)
        .with_ansi(false)
        .with_filter(env_filter());

    // Errors only — easy to alert on.
    let error_layer = fmt::layer()
        .with_target(true)
        .with_writer(err_w)
        .with_ansi(false)
        .with_filter(LevelFilter::ERROR);

    // Auth target — login/register/JWT events. Match exact `auth` target or
    // anything under `wayve::auth::*` (so module-path defaults still match).
    let auth_layer = fmt::layer()
        .with_target(true)
        .with_writer(auth_w)
        .with_ansi(false)
        .with_filter(tracing_subscriber::filter::filter_fn(|meta| {
            let t = meta.target();
            t == "auth" || t.starts_with("auth::") || t.contains("::auth")
        }));

    // Email target — sync, body_worker, OAuth, message endpoints.
    let email_layer = fmt::layer()
        .with_target(true)
        .with_writer(email_w)
        .with_ansi(false)
        .with_filter(tracing_subscriber::filter::filter_fn(|meta| {
            let t = meta.target();
            t == "email" || t.starts_with("email::") || t.contains("::email")
        }));

    registry()
        .with(stdout_layer)
        .with(app_layer)
        .with(error_layer)
        .with(auth_layer)
        .with(email_layer)
        .init();

    // Keep the appender workers alive for the whole process.
    let _ = GUARDS.set(vec![app_g, err_g, auth_g, email_g]);

    tracing::info!("logger initialised");
}

/// Convenience wrapper kept for backwards compat with existing callers in
/// `routes/auth.rs`. Emits an event on the `auth` target so it routes to
/// `auth.log` and shows up under any active request span.
pub fn log_auth(msg: impl AsRef<str>) {
    tracing::info!(target: "auth", "{}", msg.as_ref());
}

/// Custom root span for HTTP requests. Pre-declares `user_id` and
/// `wayve.entity_id` as empty fields so handler code can populate them
/// after the fact via `Span::current().record(...)`. tracing-actix-web's
/// default builder gives us request_id, method, path, status, duration —
/// we extend it with our domain-specific fields.
pub struct WayveRootSpanBuilder;

impl tracing_actix_web::RootSpanBuilder for WayveRootSpanBuilder {
    fn on_request_start(req: &actix_web::dev::ServiceRequest) -> tracing::Span {
        tracing_actix_web::root_span!(
            req,
            user_id = tracing::field::Empty,
            entity_id = tracing::field::Empty,
        )
    }

    fn on_request_end<B: actix_web::body::MessageBody>(
        span: tracing::Span,
        outcome: &Result<actix_web::dev::ServiceResponse<B>, actix_web::Error>,
    ) {
        tracing_actix_web::DefaultRootSpanBuilder::on_request_end(span, outcome);
    }
}
