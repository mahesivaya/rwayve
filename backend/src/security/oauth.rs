use crate::prelude::*;

#[derive(Debug)]
pub struct OAuthState {
    pub user_id: Option<i32>,
    pub flow: String,
}

pub async fn store_state(
    state: &str,
    user_id: Option<i32>,
    flow: &str,
    pool: &PgPool,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO oauth_states (state, user_id, flow, expires_at)
        VALUES ($1, $2, $3, NOW() + INTERVAL '10 minutes')
        "#,
    )
    .bind(state)
    .bind(user_id)
    .bind(flow)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn consume_state(state: &str, pool: &PgPool) -> Result<Option<OAuthState>> {
    let row = sqlx::query(
        r#"
        DELETE FROM oauth_states
        WHERE state = $1
          AND expires_at > NOW()
        RETURNING user_id, flow
        "#,
    )
    .bind(state)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| OAuthState {
        user_id: row.get("user_id"),
        flow: row.get("flow"),
    }))
}
