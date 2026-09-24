use sqlx::PgPool;

use crate::auth::middleware::JwksCache;
use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Config,
    pub jwks: JwksCache,
}
