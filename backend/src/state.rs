use std::sync::Arc;

use sqlx::PgPool;

use crate::{config::AppConfig, jwt::Jwt};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jwt: Jwt,
    pub http: reqwest::Client,
    pub config: Arc<AppConfig>,
}
