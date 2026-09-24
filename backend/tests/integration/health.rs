use axum::http::StatusCode;

use crate::common;

#[tokio::test]
async fn health_check_returns_200() {
    let app = common::TestApp::new().await;
    let resp = app.get("/health", None).await;
    assert_eq!(resp.status, StatusCode::OK);
}
