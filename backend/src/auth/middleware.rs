use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::AppError;
use crate::models::user::UserRole;
use crate::state::AppState;

use super::claims::ClerkClaims;

#[derive(Debug, Deserialize)]
struct JwksResponse {
    keys: Vec<JwkKey>,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct JwkKey {
    kid: String,
    n: String,
    e: String,
}

#[derive(Clone)]
pub struct JwksCache {
    pub(crate) keys: Arc<RwLock<Vec<JwkKey>>>,
    pub jwks_url: String,
}

impl JwksCache {
    pub fn new(jwks_url: String) -> Self {
        Self {
            keys: Arc::new(RwLock::new(Vec::new())),
            jwks_url,
        }
    }

    pub async fn refresh(&self) -> Result<(), AppError> {
        let resp = Client::new()
            .get(&self.jwks_url)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch JWKS: {}", e)))?
            .json::<JwksResponse>()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse JWKS: {}", e)))?;
        let mut keys = self.keys.write().await;
        *keys = resp.keys;
        Ok(())
    }

    pub async fn decode_token(&self, token: &str) -> Result<ClerkClaims, AppError> {
        let header = jsonwebtoken::decode_header(token).map_err(|_| AppError::Unauthorized)?;
        let kid = header.kid.ok_or(AppError::Unauthorized)?;

        let keys = self.keys.read().await;
        let key = keys.iter().find(|k| k.kid == kid);

        let key = match key {
            Some(k) => k.clone(),
            None => {
                drop(keys);
                self.refresh().await?;
                let keys = self.keys.read().await;
                keys.iter()
                    .find(|k| k.kid == kid)
                    .cloned()
                    .ok_or(AppError::Unauthorized)?
            }
        };

        let decoding_key = DecodingKey::from_rsa_components(&key.n, &key.e)
            .map_err(|_| AppError::Unauthorized)?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = true;

        let token_data = decode::<ClerkClaims>(token, &decoding_key, &validation)
            .map_err(|_| AppError::Unauthorized)?;

        Ok(token_data.claims)
    }
}

/// Best-effort fetch of user name from Clerk API.
async fn fetch_clerk_name(
    secret_key: &str,
    user_id: &str,
) -> (Option<String>, Option<String>) {
    let Ok(resp) = Client::new()
        .get(format!("https://api.clerk.com/v1/users/{}", user_id))
        .header("Authorization", format!("Bearer {}", secret_key))
        .send()
        .await
    else {
        return (None, None);
    };
    if !resp.status().is_success() {
        return (None, None);
    }
    let Ok(data) = resp.json::<serde_json::Value>().await else {
        return (None, None);
    };
    let first = data["first_name"].as_str().map(String::from);
    let last = data["last_name"].as_str().map(String::from);
    (first, last)
}

/// Authenticated user with role loaded from DB.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: String,
    pub role: UserRole,
    pub is_active: bool,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized)?;

        let jwks = &state.jwks;
        let claims = jwks.decode_token(token).await?;

        let user = match crate::db::users::find_by_id(&state.db, &claims.sub).await? {
            Some(u) => u,
            None => {
                // Auto-create user on first API call (handles missed webhooks)
                let (first, last) =
                    fetch_clerk_name(&state.config.clerk_secret_key, &claims.sub).await;
                crate::db::users::create(
                    &state.db,
                    &claims.sub,
                    first.as_deref(),
                    last.as_deref(),
                )
                .await?;
                crate::db::users::find_by_id(&state.db, &claims.sub)
                    .await?
                    .ok_or(AppError::Unauthorized)?
            }
        };

        Ok(AuthUser {
            id: user.id,
            role: user.role,
            is_active: user.is_active,
        })
    }
}

/// Require at least staff role.
#[derive(Debug, Clone)]
pub struct StaffUser(pub AuthUser);

impl FromRequestParts<AppState> for StaffUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = AuthUser::from_request_parts(parts, state).await?;
        match user.role {
            UserRole::Staff | UserRole::Admin => Ok(StaffUser(user)),
            _ => Err(AppError::Forbidden),
        }
    }
}

/// Require admin role.
#[derive(Debug, Clone)]
pub struct AdminUser(pub AuthUser);

impl FromRequestParts<AppState> for AdminUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = AuthUser::from_request_parts(parts, state).await?;
        match user.role {
            UserRole::Admin => Ok(AdminUser(user)),
            _ => Err(AppError::Forbidden),
        }
    }
}
