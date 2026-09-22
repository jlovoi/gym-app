use axum::{extract::FromRequestParts, http::request::Parts};
use uuid::Uuid;

use crate::{error::AppError, jwt::TokenType, state::AppState};

/// Extractor for routes that require a valid `Authorization: Bearer <access_token>` header.
pub struct AuthUser {
    pub user_id: Uuid,
    pub email: String,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        let token = header.strip_prefix("Bearer ").ok_or(AppError::Unauthorized)?;
        let claims = state.jwt.verify_session(token, TokenType::Access)?;

        Ok(AuthUser {
            user_id: claims.sub,
            email: claims.email,
        })
    }
}
