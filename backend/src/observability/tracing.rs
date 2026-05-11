use tracing_appender::rolling;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_tracing() {
    // ✅ Store logs in /logs folder
    let file_appender = rolling::daily("logs", "tracing.log");

    // ✅ Console logs
    let stdout_layer = fmt::layer()
        .with_target(false)
        .with_ansi(false) // 🔥 removes weird terminal escape codes
        .compact();

    // ✅ File logs
    let file_layer = fmt::layer().with_writer(file_appender).with_ansi(false);

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            concat!(
                "info,",
                "actix_web=info,",
                "sqlx=warn,",
                "hyper=warn,",
                "h2=warn,",
                "tokio=warn,",
                "reqwest=warn"
            )
            .into()
        }))
        .with(stdout_layer)
        .with(file_layer)
        .init();
}
