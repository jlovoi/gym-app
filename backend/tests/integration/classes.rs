use axum::http::StatusCode;
use chrono::{NaiveDate, NaiveTime};

use crate::common;

#[tokio::test]
async fn create_class_with_admin_token() {
    let app = common::TestApp::new().await;
    let user_id = "admin_create_class";
    app.seed_user(user_id, "admin", true).await;
    let token = app.token_for(user_id);

    let body = serde_json::json!({
        "date": "2025-05-01",
        "start_time": "09:00:00",
        "end_time": "10:00:00",
        "capacity": 15
    });
    let resp = app.post("/classes", &body, Some(&token)).await;
    assert_eq!(resp.status, StatusCode::OK);
    assert_eq!(resp.body["date"], "2025-05-01");
    assert_eq!(resp.body["capacity"], 15);
}

#[tokio::test]
async fn create_class_with_member_token_returns_403() {
    let app = common::TestApp::new().await;
    let user_id = "member_create_class";
    app.seed_user(user_id, "member", true).await;
    let token = app.token_for(user_id);

    let body = serde_json::json!({
        "date": "2025-05-01",
        "start_time": "09:00:00",
        "end_time": "10:00:00",
    });
    let resp = app.post("/classes", &body, Some(&token)).await;
    assert_eq!(resp.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn list_classes_by_date() {
    let app = common::TestApp::new().await;
    let user_id = "member_list_classes";
    app.seed_user(user_id, "member", true).await;
    let token = app.token_for(user_id);

    let date = NaiveDate::from_ymd_opt(2025, 5, 2).unwrap();
    let start1 = NaiveTime::from_hms_opt(8, 0, 0).unwrap();
    let end1 = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
    let start2 = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
    let end2 = NaiveTime::from_hms_opt(11, 0, 0).unwrap();

    app.seed_class(date, start1, end1, 20, None, None).await;
    app.seed_class(date, start2, end2, 15, None, None).await;

    let resp = app
        .get(&format!("/classes?date={}", date), Some(&token))
        .await;
    assert_eq!(resp.status, StatusCode::OK);
    let classes = resp.body.as_array().unwrap();
    assert_eq!(classes.len(), 2);
    assert!(classes[0].get("signup_count").is_some());
}

#[tokio::test]
async fn get_class_detail_with_members() {
    let app = common::TestApp::new().await;
    let admin_id = "admin_class_detail";
    let member_id = "member_class_detail";
    app.seed_user(admin_id, "admin", true).await;
    app.seed_user(member_id, "member", true).await;
    let token = app.token_for(member_id);

    let date = NaiveDate::from_ymd_opt(2025, 5, 3).unwrap();
    let start = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
    let end = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
    let class_id = app.seed_class(date, start, end, 20, None, None).await;

    sqlx::query("INSERT INTO class_signups (class_id, user_id) VALUES ($1, $2)")
        .bind(class_id)
        .bind(member_id)
        .execute(&app.db)
        .await
        .unwrap();

    let resp = app
        .get(&format!("/classes/{}", class_id), Some(&token))
        .await;
    assert_eq!(resp.status, StatusCode::OK);
    assert_eq!(resp.body["capacity"], 20);
    let members = resp.body["members"].as_array().unwrap();
    assert_eq!(members.len(), 1);
    assert_eq!(members[0]["user_id"], member_id);
}

#[tokio::test]
async fn signup_for_class() {
    let app = common::TestApp::new().await;
    let user_id = "member_signup";
    app.seed_user(user_id, "member", true).await;
    let token = app.token_for(user_id);

    let date = NaiveDate::from_ymd_opt(2025, 5, 4).unwrap();
    let start = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
    let end = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
    let class_id = app.seed_class(date, start, end, 20, None, None).await;

    let resp = app
        .post_empty(&format!("/classes/{}/signup", class_id), Some(&token))
        .await;
    assert_eq!(resp.status, StatusCode::OK);
    assert_eq!(resp.body["class_id"], class_id.to_string());
    assert_eq!(resp.body["user_id"], user_id);
}

#[tokio::test]
async fn signup_when_full_returns_409() {
    let app = common::TestApp::new().await;
    let date = NaiveDate::from_ymd_opt(2025, 5, 5).unwrap();
    let start = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
    let end = NaiveTime::from_hms_opt(10, 0, 0).unwrap();

    let filler = "member_filler";
    let user_id = "member_full_class";
    app.seed_user(filler, "member", true).await;
    app.seed_user(user_id, "member", true).await;

    let class_id = app.seed_class(date, start, end, 1, None, None).await;

    sqlx::query("INSERT INTO class_signups (class_id, user_id) VALUES ($1, $2)")
        .bind(class_id)
        .bind(filler)
        .execute(&app.db)
        .await
        .unwrap();

    let token = app.token_for(user_id);
    let resp = app
        .post_empty(&format!("/classes/{}/signup", class_id), Some(&token))
        .await;
    assert_eq!(resp.status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn cancel_signup() {
    let app = common::TestApp::new().await;
    let user_id = "member_cancel_signup";
    app.seed_user(user_id, "member", true).await;
    let token = app.token_for(user_id);

    let date = NaiveDate::from_ymd_opt(2025, 5, 6).unwrap();
    let start = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
    let end = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
    let class_id = app.seed_class(date, start, end, 20, None, None).await;

    sqlx::query("INSERT INTO class_signups (class_id, user_id) VALUES ($1, $2)")
        .bind(class_id)
        .bind(user_id)
        .execute(&app.db)
        .await
        .unwrap();

    let resp = app
        .delete(&format!("/classes/{}/signup", class_id), Some(&token))
        .await;
    assert_eq!(resp.status, StatusCode::OK);
    assert_eq!(resp.body["success"], true);
}

#[tokio::test]
async fn cancel_signup_when_not_signed_up_returns_404() {
    let app = common::TestApp::new().await;
    let user_id = "member_cancel_none";
    app.seed_user(user_id, "member", true).await;
    let token = app.token_for(user_id);

    let date = NaiveDate::from_ymd_opt(2025, 5, 7).unwrap();
    let start = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
    let end = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
    let class_id = app.seed_class(date, start, end, 20, None, None).await;

    let resp = app
        .delete(&format!("/classes/{}/signup", class_id), Some(&token))
        .await;
    assert_eq!(resp.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn mark_attendance() {
    let app = common::TestApp::new().await;
    let staff_id = "staff_attendance";
    let member_id = "member_attendance";
    app.seed_user(staff_id, "staff", true).await;
    app.seed_user(member_id, "member", true).await;
    let token = app.token_for(staff_id);

    let date = NaiveDate::from_ymd_opt(2025, 5, 8).unwrap();
    let start = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
    let end = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
    let class_id = app.seed_class(date, start, end, 20, None, None).await;

    sqlx::query("INSERT INTO class_signups (class_id, user_id) VALUES ($1, $2)")
        .bind(class_id)
        .bind(member_id)
        .execute(&app.db)
        .await
        .unwrap();

    let body = serde_json::json!({
        "user_ids": [member_id]
    });
    let resp = app
        .post(
            &format!("/classes/{}/attendance", class_id),
            &body,
            Some(&token),
        )
        .await;
    assert_eq!(resp.status, StatusCode::OK);
    assert_eq!(resp.body["updated"], 1);
}

#[tokio::test]
async fn mark_attendance_member_returns_403() {
    let app = common::TestApp::new().await;
    let user_id = "member_mark_att";
    app.seed_user(user_id, "member", true).await;
    let token = app.token_for(user_id);

    let date = NaiveDate::from_ymd_opt(2025, 5, 9).unwrap();
    let start = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
    let end = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
    let class_id = app.seed_class(date, start, end, 20, None, None).await;

    let body = serde_json::json!({
        "user_ids": [user_id]
    });
    let resp = app
        .post(
            &format!("/classes/{}/attendance", class_id),
            &body,
            Some(&token),
        )
        .await;
    assert_eq!(resp.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn duplicate_signup_returns_409() {
    let app = common::TestApp::new().await;
    let user_id = "member_dup_signup";
    app.seed_user(user_id, "member", true).await;
    let token = app.token_for(user_id);

    let date = NaiveDate::from_ymd_opt(2025, 5, 10).unwrap();
    let start = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
    let end = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
    let class_id = app.seed_class(date, start, end, 20, None, None).await;

    let resp = app
        .post_empty(&format!("/classes/{}/signup", class_id), Some(&token))
        .await;
    assert_eq!(resp.status, StatusCode::OK);

    let resp = app
        .post_empty(&format!("/classes/{}/signup", class_id), Some(&token))
        .await;
    assert_eq!(resp.status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn inactive_user_signup_returns_402() {
    let app = common::TestApp::new().await;
    let user_id = "member_inactive_signup";
    app.seed_user(user_id, "member", false).await;
    let token = app.token_for(user_id);

    let date = NaiveDate::from_ymd_opt(2025, 5, 11).unwrap();
    let start = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
    let end = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
    let class_id = app.seed_class(date, start, end, 20, None, None).await;

    let resp = app
        .post_empty(&format!("/classes/{}/signup", class_id), Some(&token))
        .await;
    assert_eq!(resp.status, StatusCode::PAYMENT_REQUIRED);
}

#[tokio::test]
async fn active_user_can_signup_after_activation() {
    let app = common::TestApp::new().await;
    let user_id = "member_activated_signup";
    app.seed_user(user_id, "member", false).await;
    let token = app.token_for(user_id);

    let date = NaiveDate::from_ymd_opt(2025, 5, 12).unwrap();
    let start = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
    let end = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
    let class_id = app.seed_class(date, start, end, 20, None, None).await;

    // Inactive user gets 402
    let resp = app
        .post_empty(&format!("/classes/{}/signup", class_id), Some(&token))
        .await;
    assert_eq!(resp.status, StatusCode::PAYMENT_REQUIRED);

    // Simulate activation (e.g., after Stripe checkout webhook)
    sqlx::query("UPDATE users SET is_active = true WHERE id = $1")
        .bind(user_id)
        .execute(&app.db)
        .await
        .unwrap();

    // Now signup succeeds
    let resp = app
        .post_empty(&format!("/classes/{}/signup", class_id), Some(&token))
        .await;
    assert_eq!(resp.status, StatusCode::OK);
}

#[tokio::test]
async fn list_classes_shows_user_signed_up() {
    let app = common::TestApp::new().await;
    let user_id = "member_signed_up_badge";
    app.seed_user(user_id, "member", true).await;
    let token = app.token_for(user_id);

    let date = NaiveDate::from_ymd_opt(2025, 8, 1).unwrap();
    let start = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
    let end = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
    let class_id = app.seed_class(date, start, end, 20, None, None).await;

    // Before signup: user_signed_up should be false
    let resp = app
        .get(&format!("/classes?date={}", date), Some(&token))
        .await;
    assert_eq!(resp.status, StatusCode::OK);
    let classes = resp.body.as_array().unwrap();
    assert_eq!(classes.len(), 1);
    assert_eq!(classes[0]["user_signed_up"], false);

    // Sign up
    app.post_empty(&format!("/classes/{}/signup", class_id), Some(&token))
        .await;

    // After signup: user_signed_up should be true
    let resp = app
        .get(&format!("/classes?date={}", date), Some(&token))
        .await;
    assert_eq!(resp.status, StatusCode::OK);
    let classes = resp.body.as_array().unwrap();
    assert_eq!(classes[0]["user_signed_up"], true);
}
