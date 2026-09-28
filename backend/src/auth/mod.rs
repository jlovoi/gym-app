mod apple;
pub mod extractors;
mod google;
pub mod jwt;
mod pkce;
mod routes;

use std::str::FromStr;

use crate::error::AppError;

pub use routes::router;

/// What we learn about a user from a provider after a successful code exchange.
pub struct ExternalUser {
    pub provider_user_id: String,
    /// Verified by the provider. Apple only guarantees it on the first login.
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    Google,
    Apple,
}

impl Provider {
    pub fn as_str(self) -> &'static str {
        match self {
            Provider::Google => "google",
            Provider::Apple => "apple",
        }
    }
}

impl FromStr for Provider {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "google" => Ok(Provider::Google),
            "apple" => Ok(Provider::Apple),
            other => Err(AppError::BadRequest(format!("unknown oauth provider: {other}"))),
        }
    }
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
