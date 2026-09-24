use sqlx::PgPool;

use crate::models::user::User;

pub async fn find_by_id(pool: &PgPool, id: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn create(
    pool: &PgPool,
    id: &str,
    first_name: Option<&str>,
    last_name: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO users (id, first_name, last_name) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
    )
    .bind(id)
    .bind(first_name)
    .bind(last_name)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_all(pool: &PgPool) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

pub async fn update(
    pool: &PgPool,
    id: &str,
    role: Option<&str>,
    is_active: Option<bool>,
) -> Result<Option<User>, sqlx::Error> {
    // Build dynamic update
    let user = sqlx::query_as::<_, User>(
        "UPDATE users SET
            role = COALESCE($2::user_role, role),
            is_active = COALESCE($3, is_active)
         WHERE id = $1
         RETURNING *"
    )
    .bind(id)
    .bind(role)
    .bind(is_active)
    .fetch_optional(pool)
    .await?;
    Ok(user)
}

pub async fn set_stripe_customer_id(
    pool: &PgPool,
    user_id: &str,
    stripe_customer_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET stripe_customer_id = $2 WHERE id = $1")
        .bind(user_id)
        .bind(stripe_customer_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete(pool: &PgPool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
