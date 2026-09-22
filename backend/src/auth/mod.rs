mod apple;
mod google;
mod routes;

use std::str::FromStr;

use crate::error::AppError;

pub use routes::router;

/// Info fetched from the provider after a successful code exchange, normalized to the
/// shape our own `users` table cares about.
pub struct ExternalUser {
    pub provider_user_id: String,
    pub email: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    Google,
    Apple,
}

impl FromStr for Provider {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "google" => Ok(Provider::Google),
            "apple" => Ok(Provider::Apple),
            other => Err(AppError::BadRequest(format!(
                "unknown oauth provider: {other}"
            ))),
        }
    }
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Provider::Google => "google",
            Provider::Apple => "apple",
        })
    }
}
