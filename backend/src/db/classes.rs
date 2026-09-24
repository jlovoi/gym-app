use chrono::{NaiveDate, NaiveTime};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::class::{Class, ClassMember, ClassSignup, ClassWithCount};

pub async fn list_by_date(
    pool: &PgPool,
    date: NaiveDate,
    user_id: &str,
) -> Result<Vec<ClassWithCount>, sqlx::Error> {
    sqlx::query_as::<_, ClassWithCount>(
        "SELECT c.*, COALESCE(COUNT(cs.id), 0) AS signup_count,
                EXISTS(SELECT 1 FROM class_signups WHERE class_id = c.id AND user_id = $2) AS user_signed_up
         FROM classes c
         LEFT JOIN class_signups cs ON cs.class_id = c.id
         WHERE c.date = $1
         GROUP BY c.id
         ORDER BY c.start_time",
    )
    .bind(date)
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Class>, sqlx::Error> {
    sqlx::query_as::<_, Class>("SELECT * FROM classes WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn get_members(pool: &PgPool, class_id: Uuid) -> Result<Vec<ClassMember>, sqlx::Error> {
    sqlx::query_as::<_, ClassMember>(
        "SELECT cs.user_id, cs.attended
         FROM class_signups cs
         WHERE cs.class_id = $1
         ORDER BY cs.signed_up_at"
    )
    .bind(class_id)
    .fetch_all(pool)
    .await
}

pub async fn create(
    pool: &PgPool,
    date: NaiveDate,
    start_time: NaiveTime,
    end_time: NaiveTime,
    capacity: i32,
    coach_id: Option<&str>,
    workout_id: Option<Uuid>,
) -> Result<Class, sqlx::Error> {
    sqlx::query_as::<_, Class>(
        "INSERT INTO classes (date, start_time, end_time, capacity, coach_id, workout_id)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING *"
    )
    .bind(date)
    .bind(start_time)
    .bind(end_time)
    .bind(capacity)
    .bind(coach_id)
    .bind(workout_id)
    .fetch_one(pool)
    .await
}

pub async fn signup_count(pool: &PgPool, class_id: Uuid) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM class_signups WHERE class_id = $1")
        .bind(class_id)
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

pub async fn signup(pool: &PgPool, class_id: Uuid, user_id: &str) -> Result<ClassSignup, sqlx::Error> {
    sqlx::query_as::<_, ClassSignup>(
        "INSERT INTO class_signups (class_id, user_id) VALUES ($1, $2) RETURNING *"
    )
    .bind(class_id)
    .bind(user_id)
    .fetch_one(pool)
    .await
}

pub async fn cancel_signup(pool: &PgPool, class_id: Uuid, user_id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM class_signups WHERE class_id = $1 AND user_id = $2")
        .bind(class_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn mark_attendance(
    pool: &PgPool,
    class_id: Uuid,
    user_ids: &[String],
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE class_signups SET attended = true WHERE class_id = $1 AND user_id = ANY($2)"
    )
    .bind(class_id)
    .bind(user_ids)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}
