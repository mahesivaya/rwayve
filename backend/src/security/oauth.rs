use sqlx::PgPool;

pub async fn store_state(state: &str, user_id: i32, pool: &PgPool) {
    let _ = sqlx::query(
        "INSERT INTO oauth_states (state, user_id) VALUES ($1, $2)"
    )
    .bind(state)
    .bind(user_id)
    .execute(pool)
    .await;
}