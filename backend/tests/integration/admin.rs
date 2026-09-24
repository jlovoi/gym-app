use axum::http::StatusCode;

use crate::common;

#[tokio::test]
async fn list_users_as_admin() {
    let app = common::TestApp::new().await;
    let admin_id = "admin_list_users";
    app.seed_user(admin_id, "admin", true).await;
    app.seed_user("member_a", "member", true).await;
    app.seed_user("member_b", "member", false).await;
    let token = app.token_for(admin_id);

    let resp = app.get("/admin/users", Some(&token)).await;
    assert_eq!(resp.status, StatusCode::OK);
    let users = resp.body.as_array().unwrap();
    assert!(users.len() >= 3);
}

#[tokio::test]
async fn list_users_as_member_returns_403() {
    let app = common::TestApp::new().await;
    let user_id = "member_list_users";
    app.seed_user(user_id, "member", true).await;
    let token = app.token_for(user_id);

    let resp = app.get("/admin/users", Some(&token)).await;
    assert_eq!(resp.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn list_users_as_staff_returns_403() {
    let app = common::TestApp::new().await;
    let user_id = "staff_list_users";
    app.seed_user(user_id, "staff", true).await;
    let token = app.token_for(user_id);

    let resp = app.get("/admin/users", Some(&token)).await;
    assert_eq!(resp.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn update_user_role_as_admin() {
    let app = common::TestApp::new().await;
    let admin_id = "admin_update_role";
    let target_id = "member_to_staff";
    app.seed_user(admin_id, "admin", true).await;
    app.seed_user(target_id, "member", true).await;
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
    let admin_id = "admin_update_active";
    let target_id = "member_deactivate";
    app.seed_user(admin_id, "admin", true).await;
    app.seed_user(target_id, "member", true).await;
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
    let user_id = "member_update_user";
    let target_id = "target_user";
    app.seed_user(user_id, "member", true).await;
    app.seed_user(target_id, "member", true).await;
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
    let admin_id = "admin_update_404";
    app.seed_user(admin_id, "admin", true).await;
    let token = app.token_for(admin_id);

    let body = serde_json::json!({ "role": "staff" });
    let resp = app
        .put("/admin/users/nonexistent_user_xyz", &body, Some(&token))
        .await;
    assert_eq!(resp.status, StatusCode::NOT_FOUND);
}
