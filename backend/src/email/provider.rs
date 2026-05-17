use crate::email::oauth::{refresh_access_token, try_load_google_secrets};
use crate::email::outlook::{
    OUTLOOK_MAIL_SCOPE, OutlookCredentials, outlook_credentials, refresh_outlook_token,
};
use crate::prelude::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailProvider {
    Google,
    Microsoft,
}

impl MailProvider {
    pub fn from_db(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "microsoft" | "outlook" => Self::Microsoft,
            _ => Self::Google,
        }
    }

    pub fn as_db(self) -> &'static str {
        match self {
            Self::Google => "google",
            Self::Microsoft => "microsoft",
        }
    }

    pub fn is_google(self) -> bool {
        self == Self::Google
    }

    pub fn is_microsoft(self) -> bool {
        self == Self::Microsoft
    }
}

#[derive(Clone)]
pub struct GoogleOAuthClient {
    pub client_id: String,
    pub client_secret: String,
}

pub fn google_oauth_client() -> Result<GoogleOAuthClient> {
    let secrets = try_load_google_secrets()?;
    let client_id = secrets["web"]["client_id"]
        .as_str()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| anyhow::anyhow!("client_id missing in google secrets"))?
        .to_string();
    let client_secret = secrets["web"]["client_secret"]
        .as_str()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| anyhow::anyhow!("client_secret missing in google secrets"))?
        .to_string();

    Ok(GoogleOAuthClient {
        client_id,
        client_secret,
    })
}

#[derive(Clone, Default)]
pub struct MailProviderClients {
    pub google: Option<GoogleOAuthClient>,
    pub outlook: Option<OutlookCredentials>,
}

impl MailProviderClients {
    pub fn for_providers(providers: impl IntoIterator<Item = MailProvider>) -> Self {
        let mut needs_google = false;
        let mut needs_outlook = false;

        for provider in providers {
            needs_google |= provider.is_google();
            needs_outlook |= provider.is_microsoft();
        }

        Self {
            google: if needs_google {
                google_oauth_client().ok()
            } else {
                None
            },
            outlook: if needs_outlook {
                outlook_credentials()
            } else {
                None
            },
        }
    }

    pub fn for_provider(provider: MailProvider) -> Self {
        Self::for_providers([provider])
    }
}

pub struct RefreshedEmailToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
}

pub async fn refresh_email_token(
    provider: MailProvider,
    refresh_token: &str,
    clients: MailProviderClients,
) -> Result<RefreshedEmailToken> {
    match provider {
        MailProvider::Google => {
            let google = clients
                .google
                .ok_or_else(|| anyhow::anyhow!("Google OAuth is not configured"))?;
            let access_token =
                refresh_access_token(&google.client_id, &google.client_secret, refresh_token)
                    .await?;
            Ok(RefreshedEmailToken {
                access_token,
                refresh_token: None,
            })
        }
        MailProvider::Microsoft => {
            let outlook = clients
                .outlook
                .ok_or_else(|| anyhow::anyhow!("Outlook OAuth is not configured"))?;
            let tokens = refresh_outlook_token(&outlook, refresh_token, OUTLOOK_MAIL_SCOPE).await?;
            Ok(RefreshedEmailToken {
                access_token: tokens.access_token,
                refresh_token: tokens.refresh_token,
            })
        }
    }
}

pub async fn persist_refreshed_token(
    pool: &PgPool,
    account_id: i32,
    token: &RefreshedEmailToken,
) -> Result<()> {
    sqlx::query(
        "UPDATE email_accounts
         SET access_token = $1,
             refresh_token = COALESCE(NULLIF($2, ''), refresh_token)
         WHERE id = $3",
    )
    .bind(&token.access_token)
    .bind(token.refresh_token.as_deref().unwrap_or(""))
    .bind(account_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn refresh_and_persist_email_token(
    pool: &PgPool,
    account_id: i32,
    provider: MailProvider,
    refresh_token: &str,
    clients: MailProviderClients,
) -> Result<RefreshedEmailToken> {
    let token = refresh_email_token(provider, refresh_token, clients).await?;
    persist_refreshed_token(pool, account_id, &token).await?;
    Ok(token)
}
