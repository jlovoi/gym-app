use axum::http::StatusCode;

use crate::common;

#[tokio::test]
async fn request_without_token_returns_401() {
    let app = common::TestApp::new().await;
    let resp = app.get("/workouts/today", None).await;
    assert_eq!(resp.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn request_with_invalid_token_returns_401() {
    let app = common::TestApp::new().await;
    let resp = app.get("/workouts/today", Some("not-a-valid-jwt")).await;
    assert_eq!(resp.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn request_with_valid_member_token_succeeds() {
    let app = common::TestApp::new().await;
    let user_id = "user_auth_member";
    app.seed_user(user_id, "member", true).await;
    let token = app.token_for(user_id);

    let resp = app.get("/users/me", Some(&token)).await;
    assert_eq!(resp.status, StatusCode::OK);
    assert_eq!(resp.body["user"]["id"], user_id);
}

#[tokio::test]
async fn staff_endpoint_with_member_token_returns_403() {
    let app = common::TestApp::new().await;
    let user_id = "user_auth_member2";
    app.seed_user(user_id, "member", true).await;
    let token = app.token_for(user_id);

    let body = serde_json::json!({
        "date": "2025-01-15",
        "description": "Test WOD"
    });
    let resp = app.post("/workouts", &body, Some(&token)).await;
    assert_eq!(resp.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn admin_endpoint_with_staff_token_returns_403() {
    let app = common::TestApp::new().await;
    let user_id = "user_auth_staff";
    app.seed_user(user_id, "staff", true).await;
    let token = app.token_for(user_id);

    let resp = app.get("/admin/users", Some(&token)).await;
    assert_eq!(resp.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn valid_token_auto_creates_user() {
    let app = common::TestApp::new().await;
    let token = app.token_for("user_auto_created");
    let resp = app.get("/users/me", Some(&token)).await;
    assert_eq!(resp.status, StatusCode::OK);
    assert_eq!(resp.body["user"]["id"], "user_auto_created");
    assert_eq!(resp.body["user"]["role"], "member");
    assert_eq!(resp.body["user"]["is_active"], false);
}
