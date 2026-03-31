use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct OAuthQuery {
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    pub code: String,
}