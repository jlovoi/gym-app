use axum::extract::{Path, State};
use axum::routing::{delete, get, post, put};
use axum::Json;
use axum::Router;
use uuid::Uuid;

use crate::auth::middleware::AuthUser;
use crate::error::AppError;
use crate::models::log::{CreateLog, LeaderboardEntry, Log, UpdateLog};
use crate::models::workout::ScoreType;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/workouts/{workout_id}/logs", post(create))
        .route("/workouts/{workout_id}/logs", get(leaderboard))
        .route("/logs/{id}", put(update))
        .route("/logs/{id}", delete(delete_log))
}

async fn create(
    user: AuthUser,
    State(state): State<AppState>,
    Path(workout_id): Path<Uuid>,
    Json(mut body): Json<CreateLog>,
) -> Result<Json<Log>, AppError> {
    if !user.is_active {
        return Err(AppError::PaymentRequired);
    }

    let workout = crate::db::workouts::find_by_id(&state.db, workout_id)
        .await?
        .ok_or(AppError::NotFound)?;

    // For custom scoring, compute primary_value as sum of all numeric values in data
    if workout.score_type == ScoreType::Custom {
        if let Some(ref data) = body.data {
            if let Some(obj) = data.as_object() {
                let sum: f64 = obj.values().filter_map(|v| v.as_f64()).sum();
                body.primary_value = Some(sum);
            }
        }
    }

    let log = crate::db::logs::create(
        &state.db,
        &user.id,
        workout_id,
        body.primary_value,
        body.is_rx,
        body.data.as_ref(),
    )
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) if db_err.constraint() == Some("logs_user_id_workout_id_key") => {
            AppError::Conflict("You have already logged this workout".into())
        }
        _ => AppError::Database(e),
    })?;

    Ok(Json(log))
}

async fn leaderboard(
    _user: AuthUser,
    State(state): State<AppState>,
    Path(workout_id): Path<Uuid>,
) -> Result<Json<Vec<LeaderboardEntry>>, AppError> {
    let workout = crate::db::workouts::find_by_id(&state.db, workout_id)
        .await?
        .ok_or(AppError::NotFound)?;
    let logs =
        crate::db::logs::list_by_workout(&state.db, workout_id, &workout.score_type).await?;
    Ok(Json(logs))
}

async fn update(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateLog>,
) -> Result<Json<Log>, AppError> {
    let existing = crate::db::logs::find_by_id(&state.db, id)
        .await?
        .ok_or(AppError::NotFound)?;

    if existing.user_id.as_deref() != Some(&user.id) {
        return Err(AppError::Forbidden);
    }

    let log = crate::db::logs::update(
        &state.db,
        id,
        body.primary_value,
        body.is_rx,
        body.data.as_ref(),
    )
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(log))
}

async fn delete_log(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let existing = crate::db::logs::find_by_id(&state.db, id)
        .await?
        .ok_or(AppError::NotFound)?;

    if existing.user_id.as_deref() != Some(&user.id) {
        return Err(AppError::Forbidden);
    }

    crate::db::logs::delete(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}
