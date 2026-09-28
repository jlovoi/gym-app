use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::{ExternalUser, Provider};
use crate::models::user::User;

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Resolves an OAuth login to a row in `users`, creating one on first login.
///
/// Lookup order:
/// 1. An existing identity for this provider account.
/// 2. An existing user with the same (provider-verified) email, which links the new
///    provider to that user. This is how Google and Apple logins end up as one user.
/// 3. Otherwise a new user, seeded with whatever profile info the provider gave us.
pub async fn find_or_create_for_login(
    pool: &PgPool,
    provider: Provider,
    external: &ExternalUser,
) -> Result<User, sqlx::Error> {
    let email = external.email.as_deref().map(str::to_lowercase);
    let mut tx = pool.begin().await?;

    let existing = sqlx::query_as::<_, User>(
        "SELECT u.* FROM users u
         JOIN user_identities i ON i.user_id = u.id
         WHERE i.provider = $1 AND i.provider_user_id = $2",
    )
    .bind(provider.as_str())
    .bind(&external.provider_user_id)
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(user) = existing {
        // Fill in profile fields the user doesn't have yet, but never overwrite ones
        // they've set (Apple, for instance, only sends a name on the very first login).
        let user = sqlx::query_as::<_, User>(
            "UPDATE users SET
                first_name = COALESCE(first_name, $2),
                last_name = COALESCE(last_name, $3)
             WHERE id = $1
             RETURNING *",
        )
        .bind(user.id)
        .bind(&external.first_name)
        .bind(&external.last_name)
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        return Ok(user);
    }

    let by_email = match &email {
        Some(email) => {
            sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
                .bind(email)
                .fetch_optional(&mut *tx)
                .await?
        }
        None => None,
    };

    let user = match by_email {
        Some(user) => user,
        None => {
            sqlx::query_as::<_, User>(
                "INSERT INTO users (email, first_name, last_name) VALUES ($1, $2, $3) RETURNING *",
            )
            .bind(&email)
            .bind(&external.first_name)
            .bind(&external.last_name)
            .fetch_one(&mut *tx)
            .await?
        }
    };

    sqlx::query(
        "INSERT INTO user_identities (provider, provider_user_id, user_id) VALUES ($1, $2, $3)",
    )
    .bind(provider.as_str())
    .bind(&external.provider_user_id)
    .bind(user.id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(user)
}

pub async fn list_all(pool: &PgPool) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
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

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
