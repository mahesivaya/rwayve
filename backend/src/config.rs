use dotenvy::dotenv;
use std::env;
use tracing::warn;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeRole {
    Api,
    EmailSyncWorker,
    EmailBodyWorker,
    All,
}

impl RuntimeRole {
    pub fn from_env() -> Self {
        match env::var("RWAYVE_ROLE").as_deref() {
            Ok("email-sync-worker") => Self::EmailSyncWorker,
            Ok("email-body-worker") => Self::EmailBodyWorker,
            Ok("all") => Self::All,
            _ => Self::Api,
        }
    }
}

pub fn load_env_files() {
    dotenv().ok();

    if let Ok(env_file) = env::var("ENV_FILE") {
        dotenvy::from_filename_override(env_file).ok();
    }

    let app_env = env::var("RWAYVE_ENV")
        .or_else(|_| env::var("ENV"))
        .unwrap_or_else(|_| "development".to_string());
    dotenvy::from_filename_override(format!(".env.{app_env}")).ok();
    dotenvy::from_filename_override(format!("backend/.env.{app_env}")).ok();
    dotenvy::from_filename("backend/.env").ok();

    // Centralized secrets — the single source of truth for credentials, keys,
    // and the DB connection string. Loaded last so it wins over any stale key
    // left in the config files above. In docker this file is not in the image
    // (a no-op); containers receive it through the `env_file:` directive.
    dotenvy::from_filename_override(".env.secrets").ok();
}

pub fn db_max_connections(role: RuntimeRole) -> u32 {
    if let Ok(value) = env::var("DATABASE_MAX_CONNECTIONS") {
        if let Ok(parsed) = value.parse::<u32>() {
            return parsed;
        }
        warn!(
            value,
            "Invalid DATABASE_MAX_CONNECTIONS value; using role default"
        );
    }

    match role {
        RuntimeRole::Api | RuntimeRole::All => 10,
        RuntimeRole::EmailSyncWorker | RuntimeRole::EmailBodyWorker => 5,
    }
}

pub fn listen_port() -> u16 {
    env::var("PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(8080)
}

/// Resolve the Postgres connection string.
///
/// An explicit `DATABASE_URL` always wins — docker-compose, CI, and prod set
/// it directly. Otherwise it is derived from the `POSTGRES_*` parts, so the
/// credentials are written exactly once (in `.env.secrets`) rather than also
/// being duplicated inside a `DATABASE_URL` string. `POSTGRES_HOST` defaults
/// to `localhost` for a local `cargo run`; docker sets it to `postgres_db`.
pub fn database_url() -> String {
    if let Ok(url) = env::var("DATABASE_URL") {
        let url = url.trim();
        if !url.is_empty() {
            return url.to_string();
        }
    }

    let part = |key: &str, default: &str| -> String {
        env::var(key)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| default.to_string())
    };

    let user = part("POSTGRES_USER", "wayve_user");
    let password = part("POSTGRES_PASSWORD", "");
    let host = part("POSTGRES_HOST", "localhost");
    let port = part("POSTGRES_PORT", "5432");
    let db = part("POSTGRES_DB", "wayve_dev");
    format!("postgres://{user}:{password}@{host}:{port}/{db}")
}

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    #[serial_test::serial]
    fn runtime_role_defaults_to_api() {
        unsafe { env::remove_var("RWAYVE_ROLE") };
        assert_eq!(RuntimeRole::from_env(), RuntimeRole::Api);
    }

    #[test]
    #[serial_test::serial]
    fn runtime_role_parses_known_values() {
        unsafe { env::set_var("RWAYVE_ROLE", "email-sync-worker") };
        assert_eq!(RuntimeRole::from_env(), RuntimeRole::EmailSyncWorker);
        unsafe { env::set_var("RWAYVE_ROLE", "email-body-worker") };
        assert_eq!(RuntimeRole::from_env(), RuntimeRole::EmailBodyWorker);
        unsafe { env::set_var("RWAYVE_ROLE", "all") };
        assert_eq!(RuntimeRole::from_env(), RuntimeRole::All);
        unsafe { env::remove_var("RWAYVE_ROLE") };
    }

    #[test]
    #[serial_test::serial]
    fn db_max_connections_uses_role_defaults() {
        unsafe { env::remove_var("DATABASE_MAX_CONNECTIONS") };
        assert_eq!(db_max_connections(RuntimeRole::Api), 10);
        assert_eq!(db_max_connections(RuntimeRole::EmailSyncWorker), 5);
    }

    #[test]
    #[serial_test::serial]
    fn listen_port_defaults_when_unset() {
        unsafe { env::remove_var("PORT") };
        assert_eq!(listen_port(), 8080);
    }

    #[test]
    #[serial_test::serial]
    fn listen_port_parses_and_falls_back() {
        unsafe { env::set_var("PORT", "9090") };
        assert_eq!(listen_port(), 9090);
        unsafe { env::set_var("PORT", "garbage") };
        assert_eq!(listen_port(), 8080);
    }

    #[test]
    #[serial_test::serial]
    fn database_url_prefers_explicit_value() {
        let saved = env::var("DATABASE_URL").ok();
        unsafe { env::set_var("DATABASE_URL", "postgres://x:y@h:1/db") };
        assert_eq!(database_url(), "postgres://x:y@h:1/db");
        unsafe {
            match &saved {
                Some(value) => env::set_var("DATABASE_URL", value),
                None => env::remove_var("DATABASE_URL"),
            }
        }
    }

    #[test]
    #[serial_test::serial]
    fn database_url_derives_from_postgres_parts() {
        let saved = env::var("DATABASE_URL").ok();
        unsafe {
            env::remove_var("DATABASE_URL");
            env::set_var("POSTGRES_USER", "u");
            env::set_var("POSTGRES_PASSWORD", "p");
            env::set_var("POSTGRES_HOST", "dbhost");
            env::set_var("POSTGRES_PORT", "6543");
            env::set_var("POSTGRES_DB", "mydb");
        }
        assert_eq!(database_url(), "postgres://u:p@dbhost:6543/mydb");
        unsafe {
            for key in [
                "POSTGRES_USER",
                "POSTGRES_PASSWORD",
                "POSTGRES_HOST",
                "POSTGRES_PORT",
                "POSTGRES_DB",
            ] {
                env::remove_var(key);
            }
            match &saved {
                Some(value) => env::set_var("DATABASE_URL", value),
                None => env::remove_var("DATABASE_URL"),
            }
        }
    }
}
