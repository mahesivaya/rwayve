use std::fs::{File, OpenOptions, create_dir_all};
use std::io::{self, Write};
use std::sync::{Arc, Mutex, MutexGuard};

use chrono::Local;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

const TRACING_LOG_DIR: &str = "logs";
const TRACING_LOG_PATH: &str = "logs/tracing.log";

#[derive(Clone)]
struct HourlyResetFileWriter {
    state: Arc<Mutex<HourlyResetState>>,
}

struct HourlyResetState {
    current_hour: String,
    file: File,
}

struct HourlyResetGuard<'a> {
    state: MutexGuard<'a, HourlyResetState>,
}

impl HourlyResetFileWriter {
    fn new() -> io::Result<Self> {
        create_dir_all(TRACING_LOG_DIR)?;
        let current_hour = current_hour_key();
        let file = reset_tracing_file()?;

        Ok(Self {
            state: Arc::new(Mutex::new(HourlyResetState { current_hour, file })),
        })
    }

    fn fallback() -> Self {
        let file = tempfile_file();
        Self {
            state: Arc::new(Mutex::new(HourlyResetState {
                current_hour: current_hour_key(),
                file,
            })),
        }
    }
}

impl<'a> MakeWriter<'a> for HourlyResetFileWriter {
    type Writer = HourlyResetGuard<'a>;

    fn make_writer(&'a self) -> Self::Writer {
        let mut state = self.state.lock().expect("tracing writer mutex poisoned");
        let hour = current_hour_key();

        if state.current_hour != hour {
            if let Ok(file) = reset_tracing_file() {
                state.file = file;
                state.current_hour = hour;
            }
        }

        HourlyResetGuard { state }
    }
}

impl Write for HourlyResetGuard<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.state.file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.state.file.flush()
    }
}

fn current_hour_key() -> String {
    Local::now().format("%Y-%m-%d-%H").to_string()
}

fn reset_tracing_file() -> io::Result<File> {
    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .append(true)
        .open(TRACING_LOG_PATH)
}

fn tempfile_file() -> File {
    let path = std::env::temp_dir().join(format!(
        "rwayve-tracing-fallback-{}.log",
        std::process::id()
    ));
    OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(path)
        .expect("failed to create fallback tracing file")
}

pub fn init_tracing() {
    // Keep one active tracing file and truncate it each hour so it never grows
    // across long-running local/dev sessions.
    let file_writer = HourlyResetFileWriter::new().unwrap_or_else(|e| {
        eprintln!("tracing: failed to open {TRACING_LOG_PATH}: {e}");
        HourlyResetFileWriter::fallback()
    });

    // ✅ Console logs
    let stdout_layer = fmt::layer()
        .with_target(false)
        .with_ansi(false) // 🔥 removes weird terminal escape codes
        .compact();

    // ✅ File logs
    let file_layer = fmt::layer().with_writer(file_writer).with_ansi(false);

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
