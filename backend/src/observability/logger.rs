use tracing_appender::rolling;
use tracing_subscriber::{
    EnvFilter, Layer, fmt, filter::filter_fn, layer::SubscriberExt, util::SubscriberInitExt,
};

/// Initialize the global tracing subscriber.
///
/// Layers route by `target:` so callers control where a record lands by
/// picking the target on the macro call (e.g. `tracing::info!(target: "auth", ...)`).
/// The console layer respects `RUST_LOG`; per-file layers always log at
/// their target's level so file output isn't accidentally silenced by env.
pub fn init_logger() {
    let http_access = rolling::daily("logs/http", "access.log");
    let auth_log = rolling::daily("logs/auth", "auth.log");
    let error_log = rolling::daily("logs/errors", "error.log");
    let worker_log = rolling::daily("logs/workers", "sync.log");
    let gmail_log = rolling::daily("logs/gmail", "gmail.log");
    let ws_log = rolling::daily("logs/websocket", "ws.log");
    let db_log = rolling::daily("logs/db", "db.log");
    let cache_log = rolling::daily("logs/cache", "cache.log");
    let ai_log = rolling::daily("logs/ai", "ai.log");
    let sched_log = rolling::daily("logs/scheduler", "scheduler.log");
    // Combined firehose for `tail -f` / grep during dev. Always DEBUG so the
    // file stays useful even when RUST_LOG raises the console threshold.
    let all_log = rolling::daily("logs", "all.log");
    // Human-readable dev companion. Single line per event at INFO+, easy to
    // skim while reproducing a flow.
    let dev_log = rolling::daily("logs", "dev.log");

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(fmt::layer().pretty().with_target(true).with_filter(env_filter))
        .with(
            fmt::layer()
                .with_target(true)
                .with_ansi(false)
                .with_writer(all_log)
                .with_filter(tracing_subscriber::filter::LevelFilter::DEBUG),
        )
        .with(
            fmt::layer()
                .compact()
                .with_target(true)
                .with_level(true)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_ansi(false)
                .with_writer(dev_log)
                .with_filter(tracing_subscriber::filter::LevelFilter::INFO),
        )
        .with(
            fmt::layer()
                .json()
                .with_writer(http_access)
                .with_filter(filter_fn(|m| m.target().starts_with("http"))),
        )
        .with(
            fmt::layer()
                .json()
                .with_writer(auth_log)
                .with_filter(filter_fn(|m| m.target().starts_with("auth"))),
        )
        .with(
            fmt::layer()
                .json()
                .with_writer(error_log)
                .with_filter(tracing_subscriber::filter::LevelFilter::ERROR),
        )
        .with(
            fmt::layer()
                .json()
                .with_writer(worker_log)
                .with_filter(filter_fn(|m| m.target().starts_with("worker"))),
        )
        .with(
            fmt::layer()
                .json()
                .with_writer(gmail_log)
                .with_filter(filter_fn(|m| m.target().starts_with("gmail"))),
        )
        .with(
            fmt::layer()
                .json()
                .with_writer(ws_log)
                .with_filter(filter_fn(|m| m.target().starts_with("ws"))),
        )
        .with(
            fmt::layer()
                .json()
                .with_writer(db_log)
                .with_filter(filter_fn(|m| m.target().starts_with("db"))),
        )
        .with(
            fmt::layer()
                .json()
                .with_writer(cache_log)
                .with_filter(filter_fn(|m| m.target().starts_with("cache"))),
        )
        .with(
            fmt::layer()
                .json()
                .with_writer(ai_log)
                .with_filter(filter_fn(|m| m.target().starts_with("ai"))),
        )
        .with(
            fmt::layer()
                .json()
                .with_writer(sched_log)
                .with_filter(filter_fn(|m| m.target().starts_with("sched"))),
        )
        .init();
}
