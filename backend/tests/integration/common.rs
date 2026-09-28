#![allow(dead_code)]

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use serde::Serialize;
use serde_json::Value;
use sqlx::PgPool;
use std::sync::LazyLock;
use testcontainers::runners::AsyncRunner;
use testcontainers::ImageExt;
use testcontainers_modules::postgres::Postgres;
use tower::ServiceExt;
use uuid::Uuid;

use gym_backend::auth::jwt::Jwt;
use gym_backend::config::{AuthConfig, Config, GoogleConfig};
use gym_backend::state::AppState;

// ── Shared Postgres container ───────────────────────────────────────────

static POSTGRES: LazyLock<tokio::sync::OnceCell<testcontainers::ContainerAsync<Postgres>>> =
    LazyLock::new(|| tokio::sync::OnceCell::new());

async fn get_container() -> &'static testcontainers::ContainerAsync<Postgres> {
    POSTGRES
        .get_or_init(|| async {
            Postgres::default()
                .with_tag("16-alpine")
                .start()
                .await
                .expect("Failed to start Postgres container")
        })
        .await
}

// ── Test config ─────────────────────────────────────────────────────────

fn test_config(database_url: String) -> Config {
    Config {
        database_url,
        host: "127.0.0.1".to_string(),
        port: 0,
        public_url: "http://localhost:3000".to_string(),
        auth: AuthConfig {
            jwt_secret: "test-secret-test-secret-test-secret".to_string(),
            access_token_ttl_secs: 900,
            refresh_token_ttl_secs: 3600,
            allowed_redirects: vec!["gymfrontend://auth/callback".to_string()],
            google: Some(GoogleConfig {
                client_id: "google-client-id".to_string(),
                client_secret: "google-client-secret".to_string(),
            }),
            apple: None,
        },
    }
}

// ── TestApp ─────────────────────────────────────────────────────────────

pub struct TestApp {
    pub router: Router,
    pub db: PgPool,
    pub jwt: Jwt,
}

impl TestApp {
    pub async fn new() -> Self {
        let container = get_container().await;
        let host_port = container.get_host_port_ipv4(5432).await.unwrap();
        let admin_url = format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            host_port
        );

        let db_name = format!("test_{}", Uuid::new_v4().simple());
        let admin_pool = PgPool::connect(&admin_url).await.unwrap();
        sqlx::query(&format!("CREATE DATABASE \"{}\"", db_name))
            .execute(&admin_pool)
            .await
            .expect("Failed to create test database");

        let test_url = format!(
            "postgres://postgres:postgres@127.0.0.1:{}/{}",
            host_port, db_name
        );
        let db = PgPool::connect(&test_url).await.unwrap();
        sqlx::migrate!("./migrations")
            .run(&db)
            .await
            .expect("Failed to run migrations");

        let config = test_config(test_url);
        let state = AppState::new(db.clone(), config);
        let jwt = state.jwt.clone();
        let router = gym_backend::routes::router(state);

        Self { router, db, jwt }
    }

    // ── HTTP helpers ────────────────────────────────────────────────────

    /// Sends a request as-is and returns the raw response (for redirects and headers).
    pub async fn raw(&self, req: Request<Body>) -> axum::http::Response<Body> {
        self.router.clone().oneshot(req).await.unwrap()
    }

    pub async fn get(&self, uri: &str, token: Option<&str>) -> TestResponse {
        let mut builder = Request::builder().method(Method::GET).uri(uri);
        if let Some(t) = token {
            builder = builder.header("authorization", format!("Bearer {}", t));
        }
        let req = builder.body(Body::empty()).unwrap();
        let resp = self.router.clone().oneshot(req).await.unwrap();
        TestResponse::from_response(resp).await
    }

    pub async fn post(
        &self,
        uri: &str,
        body: &impl Serialize,
        token: Option<&str>,
    ) -> TestResponse {
        let json = serde_json::to_string(body).unwrap();
        let mut builder = Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("content-type", "application/json");
        if let Some(t) = token {
            builder = builder.header("authorization", format!("Bearer {}", t));
        }
        let req = builder.body(Body::from(json)).unwrap();
        let resp = self.router.clone().oneshot(req).await.unwrap();
        TestResponse::from_response(resp).await
    }

    pub async fn post_empty(&self, uri: &str, token: Option<&str>) -> TestResponse {
        let mut builder = Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("content-type", "application/json");
        if let Some(t) = token {
            builder = builder.header("authorization", format!("Bearer {}", t));
        }
        let req = builder.body(Body::empty()).unwrap();
        let resp = self.router.clone().oneshot(req).await.unwrap();
        TestResponse::from_response(resp).await
    }

    pub async fn put(
        &self,
        uri: &str,
        body: &impl Serialize,
        token: Option<&str>,
    ) -> TestResponse {
        let json = serde_json::to_string(body).unwrap();
        let mut builder = Request::builder()
            .method(Method::PUT)
            .uri(uri)
            .header("content-type", "application/json");
        if let Some(t) = token {
            builder = builder.header("authorization", format!("Bearer {}", t));
        }
        let req = builder.body(Body::from(json)).unwrap();
        let resp = self.router.clone().oneshot(req).await.unwrap();
        TestResponse::from_response(resp).await
    }

    pub async fn delete(&self, uri: &str, token: Option<&str>) -> TestResponse {
        let mut builder = Request::builder().method(Method::DELETE).uri(uri);
        if let Some(t) = token {
            builder = builder.header("authorization", format!("Bearer {}", t));
        }
        let req = builder.body(Body::empty()).unwrap();
        let resp = self.router.clone().oneshot(req).await.unwrap();
        TestResponse::from_response(resp).await
    }

    // ── Seed helpers ────────────────────────────────────────────────────

    pub async fn seed_user(&self, role: &str, is_active: bool) -> Uuid {
        self.seed_user_with_name(role, is_active, None, None).await
    }

    pub async fn seed_user_with_name(
        &self,
        role: &str,
        is_active: bool,
        first_name: Option<&str>,
        last_name: Option<&str>,
    ) -> Uuid {
        sqlx::query_scalar(
            "INSERT INTO users (role, is_active, first_name, last_name)
             VALUES ($1::user_role, $2, $3, $4) RETURNING id",
        )
        .bind(role)
        .bind(is_active)
        .bind(first_name)
        .bind(last_name)
        .fetch_one(&self.db)
        .await
        .expect("Failed to seed user")
    }

    pub fn token_for(&self, user_id: Uuid) -> String {
        self.jwt.issue_session(user_id).unwrap().access_token
    }

    pub fn refresh_token_for(&self, user_id: Uuid) -> String {
        self.jwt.issue_session(user_id).unwrap().refresh_token
    }
}

// ── TestResponse ────────────────────────────────────────────────────────

pub struct TestResponse {
    pub status: StatusCode,
    pub body: Value,
}

impl TestResponse {
    async fn from_response(resp: axum::http::Response<Body>) -> Self {
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let body = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap_or(Value::Null)
        };
        Self { status, body }
    }
}
