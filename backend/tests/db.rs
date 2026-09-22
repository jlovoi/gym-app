//! Integration tests against a real Postgres instance. Requires DATABASE_URL to be set
//! (see .env.example / docker-compose.yml); skipped automatically otherwise.

use gym_app_backend::db;

async fn pool_or_skip() -> Option<sqlx::PgPool> {
    let Ok(url) = std::env::var("DATABASE_URL") else {
        eprintln!("skipping: DATABASE_URL not set");
        return None;
    };
    Some(db::connect(&url).await.expect("failed to connect / migrate"))
}

#[tokio::test]
async fn upsert_creates_then_updates_existing_user() {
    let Some(pool) = pool_or_skip().await else {
        return;
    };

    let provider_user_id = uuid::Uuid::new_v4().to_string();

    let created = db::upsert_oauth_user(
        &pool,
        "google",
        &provider_user_id,
        "first@example.com",
        Some("First Name"),
    )
    .await
    .unwrap();
    assert_eq!(created.email, "first@example.com");
    assert_eq!(created.name.as_deref(), Some("First Name"));

    // Logging in again with an updated email and no name (as Apple sends after the
    // first authorization) should update the email but keep the existing name, and
    // must resolve to the exact same user row.
    let updated = db::upsert_oauth_user(&pool, "google", &provider_user_id, "second@example.com", None)
        .await
        .unwrap();

    assert_eq!(updated.id, created.id);
    assert_eq!(updated.email, "second@example.com");
    assert_eq!(updated.name.as_deref(), Some("First Name"));

    let fetched = db::find_user_by_id(&pool, created.id).await.unwrap().unwrap();
    assert_eq!(fetched.email, "second@example.com");
}

#[tokio::test]
async fn same_provider_user_id_different_providers_are_distinct_users() {
    let Some(pool) = pool_or_skip().await else {
        return;
    };

    let shared_id = uuid::Uuid::new_v4().to_string();

    let google_user = db::upsert_oauth_user(&pool, "google", &shared_id, "a@example.com", None)
        .await
        .unwrap();
    let apple_user = db::upsert_oauth_user(&pool, "apple", &shared_id, "a@example.com", None)
        .await
        .unwrap();

    assert_ne!(google_user.id, apple_user.id);
}
