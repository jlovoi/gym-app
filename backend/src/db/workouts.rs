use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::workout::{ScoreType, Workout};

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Workout>, sqlx::Error> {
    sqlx::query_as::<_, Workout>("SELECT * FROM workouts WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn find_by_date(pool: &PgPool, date: NaiveDate) -> Result<Option<Workout>, sqlx::Error> {
    sqlx::query_as::<_, Workout>("SELECT * FROM workouts WHERE date = $1")
        .bind(date)
        .fetch_optional(pool)
        .await
}

pub async fn create(
    pool: &PgPool,
    date: NaiveDate,
    description: Option<&str>,
    score_type: &ScoreType,
    score_label: Option<&str>,
    score_config: Option<&serde_json::Value>,
) -> Result<Workout, sqlx::Error> {
    sqlx::query_as::<_, Workout>(
        "INSERT INTO workouts (date, description, score_type, score_label, score_config)
         VALUES ($1, $2, $3, $4, $5) RETURNING *",
    )
    .bind(date)
    .bind(description)
    .bind(score_type)
    .bind(score_label)
    .bind(score_config)
    .fetch_one(pool)
    .await
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    date: Option<NaiveDate>,
    description: Option<&str>,
    score_type: Option<&ScoreType>,
    score_label: Option<&str>,
    score_config: Option<&serde_json::Value>,
) -> Result<Option<Workout>, sqlx::Error> {
    sqlx::query_as::<_, Workout>(
        "UPDATE workouts SET
            date = COALESCE($2, date),
            description = COALESCE($3, description),
            score_type = COALESCE($4, score_type),
            score_label = COALESCE($5, score_label),
            score_config = COALESCE($6, score_config)
         WHERE id = $1
         RETURNING *",
    )
    .bind(id)
    .bind(date)
    .bind(description)
    .bind(score_type)
    .bind(score_label)
    .bind(score_config)
    .fetch_optional(pool)
    .await
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM workouts WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
