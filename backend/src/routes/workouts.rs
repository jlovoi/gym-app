use axum::extract::{Path, State};
use axum::routing::{delete, get, post, put};
use axum::Json;
use axum::Router;
use chrono::Utc;
use uuid::Uuid;

use crate::auth::middleware::{AdminUser, AuthUser, StaffUser};
use crate::error::AppError;
use crate::models::workout::{CreateWorkout, ScoreType, UpdateWorkout, Workout};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/workouts/today", get(get_today))
        .route("/workouts/{id}", get(get_by_id))
        .route("/workouts", post(create))
        .route("/workouts/{id}", put(update))
        .route("/workouts/{id}", delete(delete_workout))
}

async fn get_today(
    _user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Workout>, AppError> {
    let today = Utc::now().date_naive();
    let workout = crate::db::workouts::find_by_date(&state.db, today)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(workout))
}

async fn get_by_id(
    _user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Workout>, AppError> {
    let workout = crate::db::workouts::find_by_id(&state.db, id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(workout))
}

async fn create(
    _user: StaffUser,
    State(state): State<AppState>,
    Json(body): Json<CreateWorkout>,
) -> Result<Json<Workout>, AppError> {
    let score_type = body.score_type.unwrap_or(ScoreType::Time);
    let workout = crate::db::workouts::create(
        &state.db,
        body.date,
        body.description.as_deref(),
        &score_type,
        body.score_label.as_deref(),
        body.score_config.as_ref(),
    )
    .await?;
    Ok(Json(workout))
}

async fn update(
    _user: StaffUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateWorkout>,
) -> Result<Json<Workout>, AppError> {
    let workout = crate::db::workouts::update(
        &state.db,
        id,
        body.date,
        body.description.as_deref(),
        body.score_type.as_ref(),
        body.score_label.as_deref(),
        body.score_config.as_ref(),
    )
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(workout))
}

async fn delete_workout(
    _user: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    crate::db::workouts::delete(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}
