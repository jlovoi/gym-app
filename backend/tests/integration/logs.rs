use axum::http::StatusCode;
use chrono::NaiveDate;

use crate::common;

#[tokio::test]
async fn create_log_for_workout() {
    let app = common::TestApp::new().await;
    let user_id = "member_create_log";
    app.seed_user(user_id, "member", true).await;
    let token = app.token_for(user_id);

    let date = NaiveDate::from_ymd_opt(2025, 4, 1).unwrap();
    let workout_id = app.seed_workout(date, Some("Fran")).await;

    let body = serde_json::json!({
        "primary_value": 185.0,
        "is_rx": true,
        "data": { "notes": "PR!" }
    });
    let resp = app
        .post(
            &format!("/workouts/{}/logs", workout_id),
            &body,
            Some(&token),
        )
        .await;
    assert_eq!(resp.status, StatusCode::OK);
    assert_eq!(resp.body["user_id"], user_id);
    assert_eq!(resp.body["workout_id"], workout_id.to_string());
    assert_eq!(resp.body["primary_value"], 185.0);
    assert_eq!(resp.body["is_rx"], true);
}

#[tokio::test]
async fn leaderboard_sorted_by_primary_value() {
    let app = common::TestApp::new().await;
    let user1 = "member_lb_1";
    let user2 = "member_lb_2";
    let user3 = "member_lb_3";
    app.seed_user(user1, "member", true).await;
    app.seed_user(user2, "member", true).await;
    app.seed_user(user3, "member", true).await;

    let date = NaiveDate::from_ymd_opt(2025, 4, 2).unwrap();
    let workout_id = app.seed_workout(date, Some("Grace")).await;

    app.seed_log(user1, workout_id, Some(300.0)).await;
    app.seed_log(user2, workout_id, Some(100.0)).await;
    app.seed_log(user3, workout_id, Some(200.0)).await;

    let token = app.token_for(user1);
    let resp = app
        .get(
            &format!("/workouts/{}/logs", workout_id),
            Some(&token),
        )
        .await;
    assert_eq!(resp.status, StatusCode::OK);

    let logs = resp.body.as_array().unwrap();
    assert_eq!(logs.len(), 3);
    assert_eq!(logs[0]["primary_value"], 100.0);
    assert_eq!(logs[1]["primary_value"], 200.0);
    assert_eq!(logs[2]["primary_value"], 300.0);
}

#[tokio::test]
async fn update_own_log() {
    let app = common::TestApp::new().await;
    let user_id = "member_update_log";
    app.seed_user(user_id, "member", true).await;
    let token = app.token_for(user_id);

    let date = NaiveDate::from_ymd_opt(2025, 4, 3).unwrap();
    let workout_id = app.seed_workout(date, None).await;
    let log_id = app.seed_log(user_id, workout_id, Some(150.0)).await;

    let body = serde_json::json!({
        "primary_value": 160.0,
        "is_rx": false
    });
    let resp = app
        .put(&format!("/logs/{}", log_id), &body, Some(&token))
        .await;
    assert_eq!(resp.status, StatusCode::OK);
    assert_eq!(resp.body["primary_value"], 160.0);
    assert_eq!(resp.body["is_rx"], false);
}

#[tokio::test]
async fn update_other_users_log_returns_403() {
    let app = common::TestApp::new().await;
    let owner = "log_owner";
    let other = "log_other";
    app.seed_user(owner, "member", true).await;
    app.seed_user(other, "member", true).await;

    let date = NaiveDate::from_ymd_opt(2025, 4, 4).unwrap();
    let workout_id = app.seed_workout(date, None).await;
    let log_id = app.seed_log(owner, workout_id, Some(200.0)).await;

    let other_token = app.token_for(other);
    let body = serde_json::json!({ "primary_value": 999.0 });
    let resp = app
        .put(&format!("/logs/{}", log_id), &body, Some(&other_token))
        .await;
    assert_eq!(resp.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn delete_own_log() {
    let app = common::TestApp::new().await;
    let user_id = "member_delete_log";
    app.seed_user(user_id, "member", true).await;
    let token = app.token_for(user_id);

    let date = NaiveDate::from_ymd_opt(2025, 4, 5).unwrap();
    let workout_id = app.seed_workout(date, None).await;
    let log_id = app.seed_log(user_id, workout_id, Some(120.0)).await;

    let resp = app
        .delete(&format!("/logs/{}", log_id), Some(&token))
        .await;
    assert_eq!(resp.status, StatusCode::OK);
    assert_eq!(resp.body["deleted"], true);
}

#[tokio::test]
async fn delete_other_users_log_returns_403() {
    let app = common::TestApp::new().await;
    let owner = "log_del_owner";
    let other = "log_del_other";
    app.seed_user(owner, "member", true).await;
    app.seed_user(other, "member", true).await;

    let date = NaiveDate::from_ymd_opt(2025, 4, 6).unwrap();
    let workout_id = app.seed_workout(date, None).await;
    let log_id = app.seed_log(owner, workout_id, Some(250.0)).await;

    let other_token = app.token_for(other);
    let resp = app
        .delete(&format!("/logs/{}", log_id), Some(&other_token))
        .await;
    assert_eq!(resp.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn duplicate_log_for_same_workout_returns_409() {
    let app = common::TestApp::new().await;
    let user_id = "member_dup_log";
    app.seed_user(user_id, "member", true).await;
    let token = app.token_for(user_id);

    let date = NaiveDate::from_ymd_opt(2025, 4, 7).unwrap();
    let workout_id = app.seed_workout(date, None).await;

    let body = serde_json::json!({ "primary_value": 100.0 });

    let resp = app
        .post(
            &format!("/workouts/{}/logs", workout_id),
            &body,
            Some(&token),
        )
        .await;
    assert_eq!(resp.status, StatusCode::OK);

    let resp = app
        .post(
            &format!("/workouts/{}/logs", workout_id),
            &body,
            Some(&token),
        )
        .await;
    assert_eq!(resp.status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn create_log_for_nonexistent_workout_returns_404() {
    let app = common::TestApp::new().await;
    let user_id = "member_log_404";
    app.seed_user(user_id, "member", true).await;
    let token = app.token_for(user_id);

    let fake_id = uuid::Uuid::new_v4();
    let body = serde_json::json!({ "primary_value": 100.0 });
    let resp = app
        .post(
            &format!("/workouts/{}/logs", fake_id),
            &body,
            Some(&token),
        )
        .await;
    assert_eq!(resp.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn inactive_user_cannot_log_workout() {
    let app = common::TestApp::new().await;
    let user_id = "member_inactive_log";
    app.seed_user(user_id, "member", false).await;
    let token = app.token_for(user_id);

    let date = NaiveDate::from_ymd_opt(2025, 4, 8).unwrap();
    let workout_id = app.seed_workout(date, Some("Cindy")).await;

    let body = serde_json::json!({ "primary_value": 100.0 });
    let resp = app
        .post(
            &format!("/workouts/{}/logs", workout_id),
            &body,
            Some(&token),
        )
        .await;
    assert_eq!(resp.status, StatusCode::PAYMENT_REQUIRED);
}

#[tokio::test]
async fn leaderboard_sorted_descending_for_reps() {
    let app = common::TestApp::new().await;
    let user1 = "reps_lb_1";
    let user2 = "reps_lb_2";
    let user3 = "reps_lb_3";
    app.seed_user(user1, "member", true).await;
    app.seed_user(user2, "member", true).await;
    app.seed_user(user3, "member", true).await;

    let date = NaiveDate::from_ymd_opt(2025, 5, 1).unwrap();
    let workout_id = app.seed_workout_with_type(date, Some("Max pull-ups"), "reps").await;

    app.seed_log(user1, workout_id, Some(30.0)).await;
    app.seed_log(user2, workout_id, Some(50.0)).await;
    app.seed_log(user3, workout_id, Some(40.0)).await;

    let token = app.token_for(user1);
    let resp = app
        .get(&format!("/workouts/{}/logs", workout_id), Some(&token))
        .await;
    assert_eq!(resp.status, StatusCode::OK);

    let logs = resp.body.as_array().unwrap();
    assert_eq!(logs.len(), 3);
    // DESC order: 50, 40, 30
    assert_eq!(logs[0]["primary_value"], 50.0);
    assert_eq!(logs[1]["primary_value"], 40.0);
    assert_eq!(logs[2]["primary_value"], 30.0);
}

#[tokio::test]
async fn rounds_reps_sorted_by_data() {
    let app = common::TestApp::new().await;
    let user1 = "rr_lb_1";
    let user2 = "rr_lb_2";
    let user3 = "rr_lb_3";
    app.seed_user(user1, "member", true).await;
    app.seed_user(user2, "member", true).await;
    app.seed_user(user3, "member", true).await;

    let date = NaiveDate::from_ymd_opt(2025, 5, 2).unwrap();
    let workout_id = app
        .seed_workout_with_type(date, Some("Cindy"), "rounds_reps")
        .await;

    // user1: 12 rounds + 5 reps
    app.seed_log_with_data(
        user1,
        workout_id,
        None,
        &serde_json::json!({"rounds": 12, "extra_reps": 5}),
    )
    .await;
    // user2: 15 rounds + 0 reps (best)
    app.seed_log_with_data(
        user2,
        workout_id,
        None,
        &serde_json::json!({"rounds": 15, "extra_reps": 0}),
    )
    .await;
    // user3: 12 rounds + 10 reps
    app.seed_log_with_data(
        user3,
        workout_id,
        None,
        &serde_json::json!({"rounds": 12, "extra_reps": 10}),
    )
    .await;

    let token = app.token_for(user1);
    let resp = app
        .get(&format!("/workouts/{}/logs", workout_id), Some(&token))
        .await;
    assert_eq!(resp.status, StatusCode::OK);

    let logs = resp.body.as_array().unwrap();
    assert_eq!(logs.len(), 3);
    // DESC by rounds then extra_reps: user2 (15+0), user3 (12+10), user1 (12+5)
    assert_eq!(logs[0]["user_id"], user2);
    assert_eq!(logs[1]["user_id"], user3);
    assert_eq!(logs[2]["user_id"], user1);
}

#[tokio::test]
async fn custom_score_primary_value_is_sum() {
    let app = common::TestApp::new().await;
    let user_id = "custom_score_user";
    app.seed_user(user_id, "member", true).await;
    let token = app.token_for(user_id);

    let date = NaiveDate::from_ymd_opt(2025, 5, 3).unwrap();
    // Seed a custom workout with score_config
    let workout_id: (uuid::Uuid,) = sqlx::query_as(
        "INSERT INTO workouts (date, description, score_type, score_config) \
         VALUES ($1, $2, 'custom'::score_type, $3) RETURNING id",
    )
    .bind(date)
    .bind("Total")
    .bind(serde_json::json!([
        {"name": "Snatch", "type": "weight"},
        {"name": "C&J", "type": "weight"},
        {"name": "Back Squat", "type": "weight"}
    ]))
    .fetch_one(&app.db)
    .await
    .unwrap();
    let workout_id = workout_id.0;

    let body = serde_json::json!({
        "data": {"Snatch": 135, "C&J": 155, "Back Squat": 225}
    });
    let resp = app
        .post(
            &format!("/workouts/{}/logs", workout_id),
            &body,
            Some(&token),
        )
        .await;
    assert_eq!(resp.status, StatusCode::OK);
    // primary_value should be sum: 135 + 155 + 225 = 515
    assert_eq!(resp.body["primary_value"], 515.0);
    assert!(resp.body["data"]["Snatch"].is_number());
}

#[tokio::test]
async fn leaderboard_returns_user_names() {
    let app = common::TestApp::new().await;
    let user1 = "names_lb_1";
    let user2 = "names_lb_2";
    app.seed_user_with_name(user1, "member", true, Some("Alice"), Some("Smith"))
        .await;
    app.seed_user_with_name(user2, "member", true, Some("Bob"), None)
        .await;

    let date = NaiveDate::from_ymd_opt(2025, 5, 10).unwrap();
    let workout_id = app.seed_workout(date, Some("Test")).await;

    app.seed_log(user1, workout_id, Some(100.0)).await;
    app.seed_log(user2, workout_id, Some(200.0)).await;

    let token = app.token_for(user1);
    let resp = app
        .get(&format!("/workouts/{}/logs", workout_id), Some(&token))
        .await;
    assert_eq!(resp.status, StatusCode::OK);

    let logs = resp.body.as_array().unwrap();
    assert_eq!(logs.len(), 2);
    // Sorted ASC by default (time): 100, 200
    assert_eq!(logs[0]["first_name"], "Alice");
    assert_eq!(logs[0]["last_name"], "Smith");
    assert_eq!(logs[1]["first_name"], "Bob");
    assert!(logs[1]["last_name"].is_null());
}
