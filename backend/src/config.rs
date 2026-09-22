use std::env;

#[derive(Debug, Clone)]
pub struct GoogleConfig {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Clone)]
pub struct AppleConfig {
    pub client_id: String, // Services ID, e.g. com.example.gymapp.web
    pub team_id: String,
    pub key_id: String,
    pub private_key_pem: String,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub jwt_secret: Vec<u8>,
    pub access_token_ttl_secs: i64,
    pub refresh_token_ttl_secs: i64,
    pub backend_base_url: String,
    pub frontend_redirect_url: String,
    pub google: Option<GoogleConfig>,
    pub apple: Option<AppleConfig>,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let jwt_secret = env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set")
            .into_bytes();

        let access_token_ttl_secs = env::var("ACCESS_TOKEN_TTL_SECONDS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(900);
        let refresh_token_ttl_secs = env::var("REFRESH_TOKEN_TTL_SECONDS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60 * 60 * 24 * 30);

        let backend_base_url = env::var("BACKEND_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:3000".to_string());
        let frontend_redirect_url = env::var("FRONTEND_REDIRECT_URL")
            .unwrap_or_else(|_| "http://localhost:8081/auth/callback".to_string());

        let google = match (
            env::var("GOOGLE_CLIENT_ID"),
            env::var("GOOGLE_CLIENT_SECRET"),
        ) {
            (Ok(client_id), Ok(client_secret)) => Some(GoogleConfig {
                client_id,
                client_secret,
            }),
            _ => {
                tracing::warn!(
                    "GOOGLE_CLIENT_ID/GOOGLE_CLIENT_SECRET not set; /auth/google routes will return 501"
                );
                None
            }
        };

        let apple = match (
            env::var("APPLE_CLIENT_ID"),
            env::var("APPLE_TEAM_ID"),
            env::var("APPLE_KEY_ID"),
            env::var("APPLE_PRIVATE_KEY"),
        ) {
            (Ok(client_id), Ok(team_id), Ok(key_id), Ok(private_key_pem)) => Some(AppleConfig {
                client_id,
                team_id,
                key_id,
                // .env files can't hold real newlines in a single value, so the key is
                // commonly stored with literal "\n" sequences that need unescaping here.
                private_key_pem: private_key_pem.replace("\\n", "\n"),
            }),
            _ => {
                tracing::warn!(
                    "APPLE_CLIENT_ID/APPLE_TEAM_ID/APPLE_KEY_ID/APPLE_PRIVATE_KEY not set; /auth/apple routes will return 501"
                );
                None
            }
        };

        Self {
            database_url,
            jwt_secret,
            access_token_ttl_secs,
            refresh_token_ttl_secs,
            backend_base_url,
            frontend_redirect_url,
            google,
            apple,
        }
    }
}
