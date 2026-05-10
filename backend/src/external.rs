// Centralized base URLs for external services. Each defaults to the real
// production endpoint but can be overridden in tests (or for self-hosted
// gateways) by setting the matching env var. Keeping all overrides in one
// place makes it cheap to swap a wiremock server in for any single provider
// without scattering #[cfg(test)] branches through handlers.

pub fn google_token_url() -> String {
    std::env::var("GOOGLE_TOKEN_URL")
        .unwrap_or_else(|_| "https://oauth2.googleapis.com/token".to_string())
}

pub fn google_userinfo_url() -> String {
    std::env::var("GOOGLE_USERINFO_URL")
        .unwrap_or_else(|_| "https://www.googleapis.com/oauth2/v2/userinfo".to_string())
}

pub fn gmail_send_url() -> String {
    std::env::var("GMAIL_SEND_URL").unwrap_or_else(|_| {
        "https://gmail.googleapis.com/gmail/v1/users/me/messages/send".to_string()
    })
}

pub fn gemini_base() -> String {
    std::env::var("GEMINI_API_BASE")
        .unwrap_or_else(|_| "https://generativelanguage.googleapis.com".to_string())
}

/// Root for Gmail REST calls. The two workers (sync + body_worker) build
/// per-message URLs off this base; tests point it at a wiremock server.
pub fn gmail_api_base() -> String {
    std::env::var("GMAIL_API_BASE")
        .unwrap_or_else(|_| "https://gmail.googleapis.com".to_string())
}

pub fn zoom_oauth_token_url() -> String {
    std::env::var("ZOOM_OAUTH_TOKEN_URL")
        .unwrap_or_else(|_| "https://zoom.us/oauth/token".to_string())
}

pub fn zoom_api_base() -> String {
    std::env::var("ZOOM_API_BASE").unwrap_or_else(|_| "https://api.zoom.us".to_string())
}
