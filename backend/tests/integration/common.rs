use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use axum::Router;
use chrono::{NaiveDate, NaiveTime, Utc};
use http_body_util::BodyExt;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::Serialize;
use serde_json::Value;
use sqlx::PgPool;
use std::sync::LazyLock;
use testcontainers::runners::AsyncRunner;
use testcontainers::ImageExt;
use testcontainers_modules::postgres::Postgres;
use tower::ServiceExt;
use uuid::Uuid;

use gym_backend::auth::claims::ClerkClaims;
use gym_backend::auth::middleware::JwksCache;
use gym_backend::config::Config;
use gym_backend::state::AppState;

const TEST_RSA_PRIVATE_KEY: &str = r#"-----BEGIN PRIVATE KEY-----
MIIEvAIBADANBgkqhkiG9w0BAQEFAASCBKYwggSiAgEAAoIBAQCTbnjGFdznXJmZ
Z9lFMkP6WoSfFPKiass2FQTkpoQoO6+4EvtlyIWEF0/dQ4dceQ9+EzLtHQgcwiYN
pDXwrlNVSJPDsA0RHLpfOYr7b38FgISBcX2OD8W4Q9yI33SOqrk5Rm4Kn33X+UNX
GOY0425UaD488bw+EXxmjhzDQgPTaSovf729judVkBOWG9V6c8LOGU92eS9myinp
VP6VPoiw+JEVfNB7hmRmdc4wDU3FhBb2xjkywdaCrVGcaRfoi3k8yHR0n9/U5sxs
kfjlodm2kTMvbPDfgoyO1cW/KANJatMzm77Wty6EhIZnEJtZ+sGu7jo5DvtNUzdo
ZE/8z9yrAgMBAAECggEABqMuF8jrMZVXKvNkcjuRmOdTWjAgvlQtgulOV5bxkzgg
sJpWLB5reGduRolgQjcHDgeCPu+UKmtJOEsAzK2kfc3vLRuzcuZGs7vM+IXsA4zm
Kke4PdmS7rlLZ/QbdMLV5PC5Sowkp8EGmZi64H2uvWkWCDtf3LF/1sQmIlpJ/s4H
P72SRnMn2pyUpiPfUOOqxL4t53CPiLf6VedVymiZHmVlWRogNxYJNGbOQx44Avyg
qY7zsADBwP1EkH1XNDvA1wVgfhH1zS5pCzVEhPR0drNtICIQ8tAvAcwwPdR3GOqk
6DXT6qD4Q+cL+3cSzbYWm+dQqS6mTaOlMn4eI4DwYQKBgQDLcaoSyQNWZwzr/pgX
VbrfwYmHs4Xc2JDgleRM6Slaf0o1eKpkZyciivWxCxN0aWLsrSWjuOa/6Jusbs5x
HMZXRQYh3wLxyn1MpBTiTF0SI6t2VnHhMVvM04SUhYBqdhqgLIB8B/d7KY0Fc5RV
xRq8g+xC/XJ/XSOE504e51SoiwKBgQC5hIq+sUPUG6LCA6pQRrxM6P4W1mOOI4uU
CnTBT+eVRXZUdGRKOQ1IU6tD5EA/YKwxbfjNcXzh9SB6NQffL9BRqiwhnDYUxNoD
hoUuWl3GY5OhNTH59MQ+MXlcbrykzyvF+hJFETh8kwiFBy0ebQQWLLTIYUighylr
x2Hh/LQAYQKBgH7pWjI7yWQ2Bt6VROWRqnD5N8U6nXAmfUJM302HSi/VJkLzEkBu
BQDMdPZLtYgyUe7ZGJjouLHQ9oP737a6P5SjT28Dwr95FO8hkJGXF5xAOi8pQAM+
GklNTfCk03YWVQfEmyZEhgMD6aAT+N4EhmhBV7p2ht2jCYxYCujGYI5/AoGAWqA7
4wHXf7NsY9jEh2i2rd+X2HIsug/1LIGbHaA6IjqHnqQpJfUO3wk4ffbvx8Yi+Baf
10ScXAmSLwDe8pF585rs6hJUfPrZAaXiQ42Th8m3IaZJ3rBKeZNTlOrnrp8h5BnQ
ePr7nCd7nvitetKIj4iRW93iS8EbY/JHh80Z5aECgYBL+MDB0yy2bZeqzLS0hLF1
6J/P+OWB0+Xke+HLgSyTxX0uKo2TT8aW2AKSED2giL1Nre9a3OI2WPamHndnBMYG
nbibQd2ADOnxcOfM8Yv852iN9NzsbD9OSIM1R27iXFR5C3iIXMqVJs/k59H4Zh7W
uL8drkJNV41X+qqCXLoaeA==
-----END PRIVATE KEY-----"#;

const TEST_KID: &str = "test-key-1";
const TEST_RSA_N: &str = "k254xhXc51yZmWfZRTJD-lqEnxTyomrLNhUE5KaEKDuvuBL7ZciFhBdP3UOHXHkPfhMy7R0IHMImDaQ18K5TVUiTw7ANERy6XzmK-29_BYCEgXF9jg_FuEPciN90jqq5OUZuCp991_lDVxjmNONuVGg-PPG8PhF8Zo4cw0ID02kqL3-9vY7nVZATlhvVenPCzhlPdnkvZsop6VT-lT6IsPiRFXzQe4ZkZnXOMA1NxYQW9sY5MsHWgq1RnGkX6It5PMh0dJ_f1ObMbJH45aHZtpEzL2zw34KMjtXFvygDSWrTM5u-1rcuhISGZxCbWfrBru46OQ77TVM3aGRP_M_cqw";
const TEST_RSA_E: &str = "AQAB";

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

// ── Mock JWKS server ────────────────────────────────────────────────────

async fn create_test_jwks() -> JwksCache {
    let jwks_json = serde_json::json!({
        "keys": [{
            "kty": "RSA",
            "kid": TEST_KID,
            "use": "sig",
            "alg": "RS256",
            "n": TEST_RSA_N,
            "e": TEST_RSA_E,
        }]
    });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind mock JWKS server");
    let addr = listener.local_addr().unwrap();
    let jwks_url = format!("http://{}/jwks", addr);

    let jwks_body = jwks_json.to_string();
    tokio::spawn(async move {
        loop {
            if let Ok((stream, _)) = listener.accept().await {
                let body = jwks_body.clone();
                tokio::spawn(async move {
                    let _ = stream.readable().await;
                    let mut buf = vec![0u8; 4096];
                    let _ = stream.try_read(&mut buf);
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = stream.writable().await;
                    let _ = stream.try_write(response.as_bytes());
                });
            }
        }
    });

    let jwks = JwksCache::new(jwks_url);
    jwks.refresh().await.expect("Failed to refresh test JWKS");
    jwks
}

// ── JWT token generation ────────────────────────────────────────────────

fn make_token(user_id: &str) -> String {
    let encoding_key =
        EncodingKey::from_rsa_pem(TEST_RSA_PRIVATE_KEY.as_bytes()).expect("Invalid test RSA key");
    let now = Utc::now().timestamp() as u64;
    let claims = ClerkClaims {
        sub: user_id.to_string(),
        exp: now + 3600,
        iat: now,
        nbf: Some(now),
        iss: "https://test.clerk.accounts.dev".to_string(),
        azp: None,
    };
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(TEST_KID.to_string());
    encode(&header, &claims, &encoding_key).expect("Failed to encode JWT")
}

// ── Test config ─────────────────────────────────────────────────────────

fn test_config(database_url: String) -> Config {
    Config {
        database_url,
        clerk_jwks_url: "http://localhost:0/jwks".to_string(),
        clerk_webhook_secret: "whsec_dGVzdHNlY3JldA==".to_string(),
        stripe_secret_key: "sk_test_fake".to_string(),
        stripe_webhook_secret: "whsec_test".to_string(),
        stripe_price_unlimited: "price_unlimited_test".to_string(),
        stripe_price_punchcard: "price_punchcard_test".to_string(),
        stripe_price_dropin: "price_dropin_test".to_string(),
        clerk_secret_key: "sk_test_fake".to_string(),
        clerk_publishable_key: "pk_test_fake".to_string(),
        host: "127.0.0.1".to_string(),
        port: 0,
    }
}

// ── TestApp ─────────────────────────────────────────────────────────────

pub struct TestApp {
    pub router: Router,
    pub db: PgPool,
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

        let jwks = create_test_jwks().await;
        let config = test_config(test_url);
        let state = AppState {
            db: db.clone(),
            config,
            jwks,
        };
        let router = gym_backend::routes::router(state);

        Self { router, db }
    }

    pub fn token_for(&self, user_id: &str) -> String {
        make_token(user_id)
    }

    // ── HTTP helpers ────────────────────────────────────────────────────

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

    pub async fn seed_user(&self, id: &str, role: &str, is_active: bool) {
        sqlx::query("INSERT INTO users (id, role, is_active) VALUES ($1, $2::user_role, $3)")
            .bind(id)
            .bind(role)
            .bind(is_active)
            .execute(&self.db)
            .await
            .expect("Failed to seed user");
    }

    pub async fn seed_user_with_name(
        &self,
        id: &str,
        role: &str,
        is_active: bool,
        first_name: Option<&str>,
        last_name: Option<&str>,
    ) {
        sqlx::query(
            "INSERT INTO users (id, role, is_active, first_name, last_name) VALUES ($1, $2::user_role, $3, $4, $5)",
        )
        .bind(id)
        .bind(role)
        .bind(is_active)
        .bind(first_name)
        .bind(last_name)
        .execute(&self.db)
        .await
        .expect("Failed to seed user with name");
    }

    pub async fn seed_workout(&self, date: NaiveDate, description: Option<&str>) -> Uuid {
        let row: (Uuid,) = sqlx::query_as(
            "INSERT INTO workouts (date, description) VALUES ($1, $2) RETURNING id",
        )
        .bind(date)
        .bind(description)
        .fetch_one(&self.db)
        .await
        .expect("Failed to seed workout");
        row.0
    }

    pub async fn seed_class(
        &self,
        date: NaiveDate,
        start: NaiveTime,
        end: NaiveTime,
        capacity: i32,
        coach_id: Option<&str>,
        workout_id: Option<Uuid>,
    ) -> Uuid {
        let row: (Uuid,) = sqlx::query_as(
            "INSERT INTO classes (date, start_time, end_time, capacity, coach_id, workout_id) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
        )
        .bind(date)
        .bind(start)
        .bind(end)
        .bind(capacity)
        .bind(coach_id)
        .bind(workout_id)
        .fetch_one(&self.db)
        .await
        .expect("Failed to seed class");
        row.0
    }

    pub async fn seed_workout_with_type(
        &self,
        date: NaiveDate,
        description: Option<&str>,
        score_type: &str,
    ) -> Uuid {
        let row: (Uuid,) = sqlx::query_as(
            "INSERT INTO workouts (date, description, score_type) VALUES ($1, $2, $3::score_type) RETURNING id",
        )
        .bind(date)
        .bind(description)
        .bind(score_type)
        .fetch_one(&self.db)
        .await
        .expect("Failed to seed workout with type");
        row.0
    }

    pub async fn seed_log(
        &self,
        user_id: &str,
        workout_id: Uuid,
        primary_value: Option<f64>,
    ) -> Uuid {
        let row: (Uuid,) = sqlx::query_as(
            "INSERT INTO logs (user_id, workout_id, primary_value) VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(user_id)
        .bind(workout_id)
        .bind(primary_value)
        .fetch_one(&self.db)
        .await
        .expect("Failed to seed log");
        row.0
    }

    pub async fn seed_log_with_data(
        &self,
        user_id: &str,
        workout_id: Uuid,
        primary_value: Option<f64>,
        data: &serde_json::Value,
    ) -> Uuid {
        let row: (Uuid,) = sqlx::query_as(
            "INSERT INTO logs (user_id, workout_id, primary_value, data) VALUES ($1, $2, $3, $4) RETURNING id",
        )
        .bind(user_id)
        .bind(workout_id)
        .bind(primary_value)
        .bind(data)
        .fetch_one(&self.db)
        .await
        .expect("Failed to seed log with data");
        row.0
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
