pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod extractors;
pub mod jwt;
pub mod pkce;
pub mod state;

use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;
use serde_json::json;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{error::AppError, extractors::AuthUser, state::AppState};

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn me(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user = db::find_user_by_id(&state.db, auth.user_id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    Ok(Json(json!({
        "id": user.id,
        "email": user.email,
        "name": user.name,
        "provider": user.provider,
    })))
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/", get(|| async { "gym-app backend" }))
        .route("/health", get(health))
        .route("/me", get(me))
        .merge(auth::router())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
