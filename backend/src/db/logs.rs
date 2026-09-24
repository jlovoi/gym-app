use sqlx::PgPool;
use uuid::Uuid;

use crate::models::log::{LeaderboardEntry, Log};
use crate::models::workout::ScoreType;

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Log>, sqlx::Error> {
    sqlx::query_as::<_, Log>("SELECT * FROM logs WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn create(
    pool: &PgPool,
    user_id: &str,
    workout_id: Uuid,
    primary_value: Option<f64>,
    is_rx: Option<bool>,
    data: Option<&serde_json::Value>,
) -> Result<Log, sqlx::Error> {
    sqlx::query_as::<_, Log>(
        "INSERT INTO logs (user_id, workout_id, primary_value, is_rx, data)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING *",
    )
    .bind(user_id)
    .bind(workout_id)
    .bind(primary_value)
    .bind(is_rx)
    .bind(data)
    .fetch_one(pool)
    .await
}

pub async fn list_by_workout(
    pool: &PgPool,
    workout_id: Uuid,
    score_type: &ScoreType,
) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
    let query = format!(
        "SELECT l.*, u.first_name, u.last_name FROM logs l \
         LEFT JOIN users u ON u.id = l.user_id \
         WHERE l.workout_id = $1 {}",
        score_type.order_by_clause()
    );
    sqlx::query_as::<_, LeaderboardEntry>(&query)
        .bind(workout_id)
        .fetch_all(pool)
        .await
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    primary_value: Option<f64>,
    is_rx: Option<bool>,
    data: Option<&serde_json::Value>,
) -> Result<Option<Log>, sqlx::Error> {
    sqlx::query_as::<_, Log>(
        "UPDATE logs SET
            primary_value = COALESCE($2, primary_value),
            is_rx = COALESCE($3, is_rx),
            data = COALESCE($4, data)
         WHERE id = $1
         RETURNING *",
    )
    .bind(id)
    .bind(primary_value)
    .bind(is_rx)
    .bind(data)
    .fetch_optional(pool)
    .await
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM logs WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
