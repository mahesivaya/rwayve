use crate::email::provider::MailProvider;
use crate::prelude::*;
use sqlx::QueryBuilder;

#[derive(Clone)]
pub struct EmailAccount {
    pub id: i32,
    pub email: String,
    pub provider: MailProvider,
    pub refresh_token: Option<String>,
    pub last_sync: Option<i64>,
}

impl EmailAccount {
    pub fn usable_refresh_token(&self) -> Option<&str> {
        self.refresh_token
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
    }
}

fn account_from_row(row: sqlx::postgres::PgRow) -> EmailAccount {
    let provider = row
        .try_get::<String, _>("provider")
        .map(|value| MailProvider::from_db(&value))
        .unwrap_or(MailProvider::Google);

    EmailAccount {
        id: row.get("id"),
        email: row.try_get("email").unwrap_or_default(),
        provider,
        refresh_token: row.try_get("refresh_token").ok().flatten(),
        last_sync: row.try_get("last_sync").ok(),
    }
}

pub async fn load_email_account_for_user(
    pool: &PgPool,
    account_id: i32,
    user_id: i32,
) -> Result<Option<EmailAccount>> {
    let row = sqlx::query(
        "SELECT id, email, provider, refresh_token, last_sync
         FROM email_accounts
         WHERE id = $1 AND user_id = $2",
    )
    .bind(account_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(account_from_row))
}

pub async fn load_syncable_email_accounts(pool: &PgPool) -> Result<Vec<EmailAccount>> {
    let rows = sqlx::query(
        "SELECT id, email, provider, refresh_token, last_sync
         FROM email_accounts
         WHERE access_token IS NOT NULL",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(account_from_row).collect())
}

pub async fn load_user_email_accounts_for_older_sync(
    pool: &PgPool,
    user_id: i32,
    account_id: Option<i32>,
) -> Result<Vec<EmailAccount>> {
    let mut qb = QueryBuilder::new(
        "SELECT id, email, provider, refresh_token, last_sync
         FROM email_accounts
         WHERE user_id = ",
    );
    qb.push_bind(user_id);

    if let Some(account_id) = account_id {
        qb.push(" AND id = ");
        qb.push_bind(account_id);
    }

    let rows = qb.build().fetch_all(pool).await?;
    Ok(rows.into_iter().map(account_from_row).collect())
}

pub struct ConnectedEmailAccount<'a> {
    pub email: &'a str,
    pub user_id: i32,
    pub provider: MailProvider,
    pub access_token: &'a str,
    pub refresh_token: Option<&'a str>,
    pub expires_in: i64,
}

pub async fn upsert_connected_email_account(
    pool: &PgPool,
    account: ConnectedEmailAccount<'_>,
) -> Result<i32> {
    let expiry = (chrono::Utc::now() + chrono::Duration::seconds(account.expires_in)).naive_utc();
    let refresh_token = account.refresh_token.unwrap_or("");

    let row = sqlx::query(
        r#"
        INSERT INTO email_accounts
          (email, user_id, access_token, refresh_token, token_expiry, is_active, provider)
        VALUES ($1, $2, $3, $4, $5, true, $6)
        ON CONFLICT (user_id, email) DO UPDATE SET
          access_token = EXCLUDED.access_token,
          token_expiry = EXCLUDED.token_expiry,
          provider = EXCLUDED.provider,
          is_active = true,
          refresh_token = COALESCE(
            NULLIF(EXCLUDED.refresh_token, ''),
            email_accounts.refresh_token
          )
        RETURNING id
        "#,
    )
    .bind(account.email)
    .bind(account.user_id)
    .bind(account.access_token)
    .bind(refresh_token)
    .bind(expiry)
    .bind(account.provider.as_db())
    .fetch_one(pool)
    .await?;

    Ok(row.get("id"))
}
