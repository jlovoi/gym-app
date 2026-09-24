use axum::extract::{Path, State};
use axum::routing::{get, put};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::auth::middleware::AdminUser;
use crate::db::users as users_db;
use crate::error::AppError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/admin/users", get(list_users))
        .route("/admin/users/{id}", put(update_user))
}

async fn list_users(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    let users = users_db::list_all(&state.db).await?;
    Ok(Json(json!(users)))
}

#[derive(Deserialize)]
struct UpdateUserRequest {
    role: Option<String>,
    is_active: Option<bool>,
}

async fn update_user(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateUserRequest>,
) -> Result<Json<Value>, AppError> {
    let user = users_db::update(
        &state.db,
        &id,
        body.role.as_deref(),
        body.is_active,
    )
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(json!(user)))
}
