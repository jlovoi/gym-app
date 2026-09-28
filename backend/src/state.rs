use sqlx::PgPool;

use crate::auth::jwt::Jwt;
use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Config,
    pub jwt: Jwt,
    pub http: reqwest::Client,
}

impl AppState {
    pub fn new(db: PgPool, config: Config) -> Self {
        let jwt = Jwt::new(
            config.auth.jwt_secret.as_bytes(),
            config.auth.access_token_ttl_secs,
            config.auth.refresh_token_ttl_secs,
        );
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("failed to build HTTP client");

        Self { db, config, jwt, http }
    }
}
