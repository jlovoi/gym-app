use axum::http::StatusCode;

use crate::common;

#[tokio::test]
async fn list_users_as_admin() {
    let app = common::TestApp::new().await;
    let admin_id = app.seed_user("admin", true).await;
    app.seed_user("member", true).await;
    app.seed_user("member", false).await;
    let token = app.token_for(admin_id);

    let resp = app.get("/admin/users", Some(&token)).await;
    assert_eq!(resp.status, StatusCode::OK);
    let users = resp.body.as_array().unwrap();
    assert!(users.len() >= 3);
}

#[tokio::test]
async fn list_users_as_member_returns_403() {
    let app = common::TestApp::new().await;
    let user_id = app.seed_user("member", true).await;
    let token = app.token_for(user_id);

    let resp = app.get("/admin/users", Some(&token)).await;
    assert_eq!(resp.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn list_users_as_staff_returns_403() {
    let app = common::TestApp::new().await;
    let user_id = app.seed_user("staff", true).await;
    let token = app.token_for(user_id);

    let resp = app.get("/admin/users", Some(&token)).await;
    assert_eq!(resp.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn update_user_role_as_admin() {
    let app = common::TestApp::new().await;
    let admin_id = app.seed_user("admin", true).await;
    let target_id = app.seed_user("member", true).await;
    let token = app.token_for(admin_id);

    let body = serde_json::json!({
        "role": "staff"
    });
    let resp = app
        .put(&format!("/admin/users/{}", target_id), &body, Some(&token))
        .await;
    assert_eq!(resp.status, StatusCode::OK);
    assert_eq!(resp.body["role"], "staff");
}

#[tokio::test]
async fn update_user_active_status() {
    let app = common::TestApp::new().await;
    let admin_id = app.seed_user("admin", true).await;
    let target_id = app.seed_user("member", true).await;
    let token = app.token_for(admin_id);

    let body = serde_json::json!({
        "is_active": false
    });
    let resp = app
        .put(&format!("/admin/users/{}", target_id), &body, Some(&token))
        .await;
    assert_eq!(resp.status, StatusCode::OK);
    assert_eq!(resp.body["is_active"], false);
}

#[tokio::test]
async fn update_user_as_member_returns_403() {
    let app = common::TestApp::new().await;
    let user_id = app.seed_user("member", true).await;
    let target_id = app.seed_user("member", true).await;
    let token = app.token_for(user_id);

    let body = serde_json::json!({ "role": "admin" });
    let resp = app
        .put(&format!("/admin/users/{}", target_id), &body, Some(&token))
        .await;
    assert_eq!(resp.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn update_nonexistent_user_returns_404() {
    let app = common::TestApp::new().await;
    let admin_id = app.seed_user("admin", true).await;
    let token = app.token_for(admin_id);

    let body = serde_json::json!({ "role": "staff" });
    let resp = app
        .put(&format!("/admin/users/{}", uuid::Uuid::new_v4()), &body, Some(&token))
        .await;
    assert_eq!(resp.status, StatusCode::NOT_FOUND);
}
