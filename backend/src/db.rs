use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;

use crate::error::AppError;

pub async fn connect(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub provider: String,
    pub provider_user_id: String,
    pub email: String,
    pub name: Option<String>,
}

pub async fn upsert_oauth_user(
    pool: &PgPool,
    provider: &str,
    provider_user_id: &str,
    email: &str,
    name: Option<&str>,
) -> Result<User, AppError> {
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (provider, provider_user_id, email, name)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (provider, provider_user_id)
        DO UPDATE SET
            email = EXCLUDED.email,
            name = COALESCE(EXCLUDED.name, users.name),
            updated_at = now()
        RETURNING id, provider, provider_user_id, email, name
        "#,
    )
    .bind(provider)
    .bind(provider_user_id)
    .bind(email)
    .bind(name)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn find_user_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, provider, provider_user_id, email, name FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}
