use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use uuid::Uuid;

use crate::auth::jwt::TokenType;
use crate::db::users as users_db;
use crate::error::AppError;
use crate::models::user::{User, UserRole};
use crate::state::AppState;

/// Any request with a valid `Authorization: Bearer <access_token>` header.
/// Doesn't touch the database; use `CurrentUser` when you need the row.
pub struct AuthUser {
    pub id: Uuid,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, AppError> {
        let token = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or(AppError::Unauthorized)?;

        let claims = state.jwt.verify_session(token, TokenType::Access)?;
        Ok(AuthUser { id: claims.sub })
    }
}

/// An authenticated user, loaded from the database. Rejects tokens for deleted users.
pub struct CurrentUser(pub User);

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, AppError> {
        let auth = AuthUser::from_request_parts(parts, state).await?;
        let user = users_db::find_by_id(&state.db, auth.id)
            .await?
            .ok_or(AppError::Unauthorized)?;
        Ok(CurrentUser(user))
    }
}

/// An authenticated admin. The role is read from the database on every request, so
/// promotions and demotions take effect immediately rather than when the token expires.
pub struct AdminUser(pub User);

impl FromRequestParts<AppState> for AdminUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, AppError> {
        let CurrentUser(user) = CurrentUser::from_request_parts(parts, state).await?;
        if user.role != UserRole::Admin {
            return Err(AppError::Forbidden);
        }
        Ok(AdminUser(user))
    }
}
