use std::env;
use std::fmt;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    /// Public base URL of this backend, used to build OAuth callback URLs,
    /// e.g. `https://gym-backend-ak8mug.fly.dev`.
    pub public_url: String,
    pub auth: AuthConfig,
}

#[derive(Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub access_token_ttl_secs: i64,
    pub refresh_token_ttl_secs: i64,
    /// Exact-match allowlist of where the backend may send users (and their tokens)
    /// after login. Anything else is rejected to avoid leaking tokens to arbitrary URLs.
    pub allowed_redirects: Vec<String>,
    pub google: Option<GoogleConfig>,
    pub apple: Option<AppleConfig>,
}

#[derive(Clone)]
pub struct GoogleConfig {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Clone)]
pub struct AppleConfig {
    /// The Services ID (not the App ID) configured for Sign in with Apple on the web.
    pub client_id: String,
    pub team_id: String,
    pub key_id: String,
    pub private_key_pem: String,
}

// Hand-written so secrets never end up in logs via `{:?}`.
impl fmt::Debug for AuthConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuthConfig")
            .field("access_token_ttl_secs", &self.access_token_ttl_secs)
            .field("refresh_token_ttl_secs", &self.refresh_token_ttl_secs)
            .field("allowed_redirects", &self.allowed_redirects)
            .field("google", &self.google.is_some())
            .field("apple", &self.apple.is_some())
            .finish_non_exhaustive()
    }
}

impl Config {
    pub fn from_env() -> Self {
        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".into())
            .parse()
            .expect("PORT must be a number");

        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port,
            public_url: env::var("PUBLIC_URL")
                .unwrap_or_else(|_| format!("http://localhost:{port}"))
                .trim_end_matches('/')
                .to_string(),
            auth: AuthConfig::from_env(),
        }
    }
}

impl AuthConfig {
    fn from_env() -> Self {
        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        assert!(jwt_secret.len() >= 32, "JWT_SECRET must be at least 32 bytes");

        let google = match (env::var("GOOGLE_CLIENT_ID"), env::var("GOOGLE_CLIENT_SECRET")) {
            (Ok(client_id), Ok(client_secret)) => Some(GoogleConfig { client_id, client_secret }),
            _ => None,
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
                // Allow the PEM to be supplied on one line with literal "\n"s (e.g. fly secrets).
                private_key_pem: private_key_pem.replace("\\n", "\n"),
            }),
            _ => None,
        };

        if google.is_none() && apple.is_none() {
            tracing::warn!("No OAuth providers configured; nobody will be able to log in");
        }

        Self {
            jwt_secret,
            access_token_ttl_secs: parse_or("ACCESS_TOKEN_TTL_SECS", 15 * 60),
            refresh_token_ttl_secs: parse_or("REFRESH_TOKEN_TTL_SECS", 30 * 24 * 60 * 60),
            allowed_redirects: env::var("AUTH_ALLOWED_REDIRECTS")
                .unwrap_or_else(|_| "gymfrontend://auth/callback".into())
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            google,
            apple,
        }
    }
}

fn parse_or(key: &str, default: i64) -> i64 {
    env::var(key)
        .ok()
        .map(|v| v.parse().unwrap_or_else(|_| panic!("{key} must be a number")))
        .unwrap_or(default)
}
