use sqlx::PgPool;

use crate::models::membership::Membership;

pub async fn find_active_by_user(
    pool: &PgPool,
    user_id: &str,
) -> Result<Option<Membership>, sqlx::Error> {
    sqlx::query_as::<_, Membership>(
        "SELECT * FROM memberships WHERE user_id = $1 AND status = 'active' ORDER BY created_at DESC LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn upsert_from_checkout(
    pool: &PgPool,
    user_id: &str,
    stripe_subscription_id: &str,
    plan_type: &str,
) -> Result<Membership, sqlx::Error> {
    sqlx::query_as::<_, Membership>(
        "INSERT INTO memberships (user_id, stripe_subscription_id, plan_type, status)
         VALUES ($1, $2, $3, 'active')
         ON CONFLICT (stripe_subscription_id) DO UPDATE
           SET status = 'active', plan_type = $3, updated_at = now()
         RETURNING *",
    )
    .bind(user_id)
    .bind(stripe_subscription_id)
    .bind(plan_type)
    .fetch_one(pool)
    .await
}

pub async fn update_status_by_subscription(
    pool: &PgPool,
    stripe_subscription_id: &str,
    status: &str,
) -> Result<Option<Membership>, sqlx::Error> {
    sqlx::query_as::<_, Membership>(
        "UPDATE memberships
         SET status = $2::membership_status, updated_at = now()
         WHERE stripe_subscription_id = $1
         RETURNING *",
    )
    .bind(stripe_subscription_id)
    .bind(status)
    .fetch_optional(pool)
    .await
}
