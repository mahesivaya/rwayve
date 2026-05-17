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
}
