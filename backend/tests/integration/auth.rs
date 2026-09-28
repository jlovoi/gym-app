use axum::body::Body;
use axum::http::header::{COOKIE, LOCATION, SET_COOKIE};
use axum::http::{Request, StatusCode};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};

use gym_backend::auth::{ExternalUser, Provider};
use gym_backend::db::users as users_db;

use crate::common;

const APP_REDIRECT: &str = "gymfrontend://auth/callback";

fn get(uri: &str) -> Request<Body> {
    Request::builder().uri(uri).body(Body::empty()).unwrap()
}

fn query_param(url: &str, key: &str) -> Option<String> {
    reqwest::Url::parse(url)
        .unwrap()
        .query_pairs()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.into_owned())
}

/// Starts a Google login and returns (signed state, PKCE cookie as "name=value").
async fn start_google(app: &common::TestApp) -> (String, String) {
    let resp = app
        .raw(get(&format!("/auth/google/start?redirect_to={APP_REDIRECT}")))
        .await;
    assert_eq!(resp.status(), StatusCode::SEE_OTHER);

    let location = resp.headers()[LOCATION].to_str().unwrap().to_string();
    let cookie = resp.headers()[SET_COOKIE].to_str().unwrap();
    let cookie = cookie.split(';').next().unwrap().to_string();
    (query_param(&location, "state").unwrap(), cookie)
}

fn external(sub: &str, email: Option<&str>, first: Option<&str>) -> ExternalUser {
    ExternalUser {
        provider_user_id: sub.into(),
        email: email.map(Into::into),
        first_name: first.map(Into::into),
        last_name: None,
    }
}

// ── /auth/{provider}/start ──────────────────────────────────────────────

#[tokio::test]
async fn start_redirects_to_google_with_pkce_and_state() {
    let app = common::TestApp::new().await;
    let resp = app
        .raw(get(&format!("/auth/google/start?redirect_to={APP_REDIRECT}")))
        .await;

    assert_eq!(resp.status(), StatusCode::SEE_OTHER);
    let location = resp.headers()[LOCATION].to_str().unwrap();
    assert!(location.starts_with("https://accounts.google.com/"));
    assert_eq!(query_param(location, "client_id").unwrap(), "google-client-id");
    assert_eq!(
        query_param(location, "redirect_uri").unwrap(),
        "http://localhost:3000/auth/google/callback"
    );
    assert_eq!(query_param(location, "code_challenge_method").unwrap(), "S256");
    assert!(query_param(location, "state").is_some());

    let cookie = resp.headers()[SET_COOKIE].to_str().unwrap();
    assert!(cookie.starts_with("oauth_pkce="));
    assert!(cookie.contains("HttpOnly"));
}

#[tokio::test]
async fn pkce_verifier_is_only_in_the_cookie() {
    let app = common::TestApp::new().await;
    let resp = app
        .raw(get(&format!("/auth/google/start?redirect_to={APP_REDIRECT}")))
        .await;

    let location = resp.headers()[LOCATION].to_str().unwrap();
    let cookie = resp.headers()[SET_COOKIE].to_str().unwrap();
    let verifier = cookie
        .split(';')
        .next()
        .unwrap()
        .strip_prefix("oauth_pkce=")
        .unwrap();

    // `state` is readable by anyone who sees the URL, so check its decoded payload too.
    let state = query_param(location, "state").unwrap();
    let payload = state.split('.').nth(1).unwrap();
    let payload = String::from_utf8(URL_SAFE_NO_PAD.decode(payload).unwrap()).unwrap();

    assert!(!location.contains(verifier));
    assert!(!payload.contains(verifier));
    assert_eq!(
        query_param(location, "code_challenge").unwrap(),
        URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
    );
}

#[tokio::test]
async fn start_rejects_unlisted_redirect() {
    let app = common::TestApp::new().await;
    let resp = app
        .get("/auth/google/start?redirect_to=https://evil.example/steal", None)
        .await;
    assert_eq!(resp.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn start_unconfigured_provider_is_404() {
    let app = common::TestApp::new().await;
    let resp = app.get("/auth/apple/start", None).await;
    assert_eq!(resp.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn start_unknown_provider_is_400() {
    let app = common::TestApp::new().await;
    let resp = app.get("/auth/myspace/start", None).await;
    assert_eq!(resp.status, StatusCode::BAD_REQUEST);
}

// ── /auth/google/callback ───────────────────────────────────────────────

#[tokio::test]
async fn callback_rejects_forged_state() {
    let app = common::TestApp::new().await;
    let resp = app
        .get("/auth/google/callback?state=not-a-jwt&code=abc", None)
        .await;
    assert_eq!(resp.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn callback_rejects_state_without_matching_cookie() {
    let app = common::TestApp::new().await;
    let (state, _cookie) = start_google(&app).await;

    let resp = app
        .get(&format!("/auth/google/callback?state={state}&code=abc"), None)
        .await;
    assert_eq!(resp.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn callback_rejects_cookie_from_another_login() {
    let app = common::TestApp::new().await;
    let (state, _) = start_google(&app).await;
    let (_, other_cookie) = start_google(&app).await;

    let req = Request::builder()
        .uri(format!("/auth/google/callback?state={state}&error=access_denied"))
        .header(COOKIE, other_cookie)
        .body(Body::empty())
        .unwrap();
    assert_eq!(app.raw(req).await.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn callback_with_provider_error_redirects_back_to_app() {
    let app = common::TestApp::new().await;
    let (state, cookie) = start_google(&app).await;

    let req = Request::builder()
        .uri(format!("/auth/google/callback?state={state}&error=access_denied"))
        .header(COOKIE, cookie)
        .body(Body::empty())
        .unwrap();
    let resp = app.raw(req).await;

    assert_eq!(resp.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        resp.headers()[LOCATION].to_str().unwrap(),
        format!("{APP_REDIRECT}#error=access_denied")
    );
}

// ── /auth/refresh and authenticated routes ──────────────────────────────

#[tokio::test]
async fn refresh_issues_new_tokens() {
    let app = common::TestApp::new().await;
    let user_id = app.seed_user("member", true).await;
    let body = serde_json::json!({ "refresh_token": app.refresh_token_for(user_id) });

    let resp = app.post("/auth/refresh", &body, None).await;
    assert_eq!(resp.status, StatusCode::OK);
    assert_eq!(resp.body["token_type"], "Bearer");

    let access = resp.body["access_token"].as_str().unwrap();
    let me = app.get("/users/me", Some(access)).await;
    assert_eq!(me.status, StatusCode::OK);
    assert_eq!(me.body["id"], user_id.to_string());
}

#[tokio::test]
async fn refresh_rejects_access_token() {
    let app = common::TestApp::new().await;
    let user_id = app.seed_user("member", true).await;
    let body = serde_json::json!({ "refresh_token": app.token_for(user_id) });

    let resp = app.post("/auth/refresh", &body, None).await;
    assert_eq!(resp.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn refresh_rejects_deleted_user() {
    let app = common::TestApp::new().await;
    let user_id = app.seed_user("member", true).await;
    let refresh_token = app.refresh_token_for(user_id);
    users_db::delete(&app.db, user_id).await.unwrap();

    let body = serde_json::json!({ "refresh_token": refresh_token });
    let resp = app.post("/auth/refresh", &body, None).await;
    assert_eq!(resp.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn me_requires_a_token() {
    let app = common::TestApp::new().await;
    let resp = app.get("/users/me", None).await;
    assert_eq!(resp.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn me_rejects_refresh_token() {
    let app = common::TestApp::new().await;
    let user_id = app.seed_user("member", true).await;
    let resp = app.get("/users/me", Some(&app.refresh_token_for(user_id))).await;
    assert_eq!(resp.status, StatusCode::UNAUTHORIZED);
}

// ── Mapping provider logins onto `users` ────────────────────────────────

#[tokio::test]
async fn first_login_creates_user_with_profile() {
    let app = common::TestApp::new().await;
    let user = users_db::find_or_create_for_login(
        &app.db,
        Provider::Google,
        &external("g-1", Some("Jo@Example.com"), Some("Jo")),
    )
    .await
    .unwrap();

    assert_eq!(user.email.as_deref(), Some("jo@example.com"));
    assert_eq!(user.first_name.as_deref(), Some("Jo"));
    assert!(!user.is_active);
}

#[tokio::test]
async fn repeat_login_returns_same_user_and_keeps_existing_name() {
    let app = common::TestApp::new().await;
    let first = users_db::find_or_create_for_login(
        &app.db,
        Provider::Google,
        &external("g-2", Some("sam@example.com"), Some("Sam")),
    )
    .await
    .unwrap();
    let second = users_db::find_or_create_for_login(
        &app.db,
        Provider::Google,
        &external("g-2", Some("sam@example.com"), Some("Samuel")),
    )
    .await
    .unwrap();

    assert_eq!(first.id, second.id);
    assert_eq!(second.first_name.as_deref(), Some("Sam"));
}

#[tokio::test]
async fn apple_login_backfills_missing_name() {
    let app = common::TestApp::new().await;
    let first = users_db::find_or_create_for_login(&app.db, Provider::Apple, &external("a-1", None, None))
        .await
        .unwrap();
    let second =
        users_db::find_or_create_for_login(&app.db, Provider::Apple, &external("a-1", None, Some("Ari")))
            .await
            .unwrap();

    assert_eq!(first.id, second.id);
    assert_eq!(second.first_name.as_deref(), Some("Ari"));
}

#[tokio::test]
async fn different_provider_with_same_email_links_to_existing_user() {
    let app = common::TestApp::new().await;
    let google = users_db::find_or_create_for_login(
        &app.db,
        Provider::Google,
        &external("g-3", Some("pat@example.com"), None),
    )
    .await
    .unwrap();
    let apple = users_db::find_or_create_for_login(
        &app.db,
        Provider::Apple,
        &external("a-3", Some("PAT@example.com"), None),
    )
    .await
    .unwrap();

    assert_eq!(google.id, apple.id);
    let identities: i64 = sqlx::query_scalar("SELECT count(*) FROM user_identities WHERE user_id = $1")
        .bind(google.id)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(identities, 2);
}

#[tokio::test]
async fn logins_without_email_are_separate_users() {
    let app = common::TestApp::new().await;
    let a = users_db::find_or_create_for_login(&app.db, Provider::Apple, &external("a-4", None, None))
        .await
        .unwrap();
    let b = users_db::find_or_create_for_login(&app.db, Provider::Apple, &external("a-5", None, None))
        .await
        .unwrap();

    assert_ne!(a.id, b.id);
}
