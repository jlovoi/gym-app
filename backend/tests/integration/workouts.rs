use axum::http::StatusCode;
use chrono::{NaiveDate, Utc};

use crate::common;

#[tokio::test]
async fn create_workout_with_staff_token() {
    let app = common::TestApp::new().await;
    let user_id = "staff_create_workout";
    app.seed_user(user_id, "staff", true).await;
    let token = app.token_for(user_id);

    let body = serde_json::json!({
        "date": "2025-06-15",
        "description": "21-15-9 Thrusters & Pull-ups (Fran)"
    });
    let resp = app.post("/workouts", &body, Some(&token)).await;
    assert_eq!(resp.status, StatusCode::OK);
    assert_eq!(resp.body["date"], "2025-06-15");
    assert_eq!(
        resp.body["description"],
        "21-15-9 Thrusters & Pull-ups (Fran)"
    );
    assert!(resp.body["id"].is_string());
}

#[tokio::test]
async fn get_today_workout() {
    let app = common::TestApp::new().await;
    let user_id = "member_get_today";
    app.seed_user(user_id, "member", true).await;
    let token = app.token_for(user_id);

    let today = Utc::now().date_naive();
    app.seed_workout(today, Some("Today's WOD")).await;

    let resp = app.get("/workouts/today", Some(&token)).await;
    assert_eq!(resp.status, StatusCode::OK);
    assert_eq!(resp.body["date"], today.to_string());
    assert_eq!(resp.body["description"], "Today's WOD");
}

#[tokio::test]
async fn get_today_workout_returns_404_when_none() {
    let app = common::TestApp::new().await;
    let user_id = "member_no_today";
    app.seed_user(user_id, "member", true).await;
    let token = app.token_for(user_id);

    let resp = app.get("/workouts/today", Some(&token)).await;
    assert_eq!(resp.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_workout_by_id() {
    let app = common::TestApp::new().await;
    let user_id = "member_get_by_id";
    app.seed_user(user_id, "member", true).await;
    let token = app.token_for(user_id);

    let date = NaiveDate::from_ymd_opt(2025, 3, 10).unwrap();
    let workout_id = app.seed_workout(date, Some("Murph")).await;

    let resp = app
        .get(&format!("/workouts/{}", workout_id), Some(&token))
        .await;
    assert_eq!(resp.status, StatusCode::OK);
    assert_eq!(resp.body["description"], "Murph");
}

#[tokio::test]
async fn update_workout_with_staff_token() {
    let app = common::TestApp::new().await;
    let user_id = "staff_update_workout";
    app.seed_user(user_id, "staff", true).await;
    let token = app.token_for(user_id);

    let date = NaiveDate::from_ymd_opt(2025, 3, 11).unwrap();
    let workout_id = app.seed_workout(date, Some("Old description")).await;

    let body = serde_json::json!({
        "description": "Updated description"
    });
    let resp = app
        .put(&format!("/workouts/{}", workout_id), &body, Some(&token))
        .await;
    assert_eq!(resp.status, StatusCode::OK);
    assert_eq!(resp.body["description"], "Updated description");
}

#[tokio::test]
async fn delete_workout_with_admin_token() {
    let app = common::TestApp::new().await;
    let user_id = "admin_delete_workout";
    app.seed_user(user_id, "admin", true).await;
    let token = app.token_for(user_id);

    let date = NaiveDate::from_ymd_opt(2025, 3, 12).unwrap();
    let workout_id = app.seed_workout(date, Some("To be deleted")).await;

    let resp = app
        .delete(&format!("/workouts/{}", workout_id), Some(&token))
        .await;
    assert_eq!(resp.status, StatusCode::OK);
    assert_eq!(resp.body["deleted"], true);
}

#[tokio::test]
async fn delete_workout_with_member_token_returns_403() {
    let app = common::TestApp::new().await;
    let user_id = "member_delete_workout";
    app.seed_user(user_id, "member", true).await;
    let token = app.token_for(user_id);

    let date = NaiveDate::from_ymd_opt(2025, 3, 13).unwrap();
    let workout_id = app.seed_workout(date, Some("Protected")).await;

    let resp = app
        .delete(&format!("/workouts/{}", workout_id), Some(&token))
        .await;
    assert_eq!(resp.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn delete_workout_with_staff_token_returns_403() {
    let app = common::TestApp::new().await;
    let user_id = "staff_delete_workout";
    app.seed_user(user_id, "staff", true).await;
    let token = app.token_for(user_id);

    let date = NaiveDate::from_ymd_opt(2025, 3, 14).unwrap();
    let workout_id = app.seed_workout(date, None).await;

    let resp = app
        .delete(&format!("/workouts/{}", workout_id), Some(&token))
        .await;
    assert_eq!(resp.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn create_workout_with_score_type() {
    let app = common::TestApp::new().await;
    let user_id = "staff_score_type";
    app.seed_user(user_id, "staff", true).await;
    let token = app.token_for(user_id);

    let body = serde_json::json!({
        "date": "2025-07-01",
        "description": "Max deadlift",
        "score_type": "weight",
        "score_label": "lbs"
    });
    let resp = app.post("/workouts", &body, Some(&token)).await;
    assert_eq!(resp.status, StatusCode::OK);
    assert_eq!(resp.body["score_type"], "weight");
    assert_eq!(resp.body["score_label"], "lbs");
}

#[tokio::test]
async fn create_workout_defaults_to_time() {
    let app = common::TestApp::new().await;
    let user_id = "staff_default_time";
    app.seed_user(user_id, "staff", true).await;
    let token = app.token_for(user_id);

    let body = serde_json::json!({
        "date": "2025-07-02",
        "description": "Fran"
    });
    let resp = app.post("/workouts", &body, Some(&token)).await;
    assert_eq!(resp.status, StatusCode::OK);
    assert_eq!(resp.body["score_type"], "time");
}
