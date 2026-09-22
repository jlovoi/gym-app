use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("unauthorized")]
    Unauthorized,
    #[error("{0}")]
    BadRequest(String),
    #[error("{0} oauth is not configured on this server")]
    ProviderNotConfigured(String),
    #[error("{0}")]
    Upstream(String),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::ProviderNotConfigured(_) => (StatusCode::NOT_IMPLEMENTED, self.to_string()),
            AppError::Upstream(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
            AppError::Database(err) => {
                tracing::error!(error = %err, "database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                )
            }
            AppError::Jwt(err) => {
                tracing::warn!(error = %err, "jwt error");
                (StatusCode::UNAUTHORIZED, "invalid token".to_string())
            }
            AppError::Http(err) => {
                tracing::error!(error = %err, "http client error");
                (
                    StatusCode::BAD_GATEWAY,
                    "upstream request failed".to_string(),
                )
            }
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}
