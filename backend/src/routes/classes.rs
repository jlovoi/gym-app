use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::NaiveDate;
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::middleware::{AdminUser, AuthUser, StaffUser};
use crate::error::AppError;
use crate::models::class::ClassDetail;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/classes", get(list_classes).post(create_class))
        .route("/classes/{id}", get(get_class))
        .route(
            "/classes/{id}/signup",
            post(signup_for_class).delete(cancel_signup),
        )
        .route("/classes/{id}/attendance", post(mark_attendance))
}

#[derive(Deserialize)]
struct ListClassesQuery {
    date: Option<NaiveDate>,
}

async fn list_classes(
    user: AuthUser,
    State(state): State<AppState>,
    Query(params): Query<ListClassesQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let date = params.date.unwrap_or_else(|| chrono::Utc::now().date_naive());
    let classes = crate::db::classes::list_by_date(&state.db, date, &user.id).await?;
    Ok(Json(serde_json::json!(classes)))
}

async fn get_class(
    _user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ClassDetail>, AppError> {
    let class = crate::db::classes::find_by_id(&state.db, id)
        .await?
        .ok_or(AppError::NotFound)?;
    let members = crate::db::classes::get_members(&state.db, id).await?;
    Ok(Json(ClassDetail {
        id: class.id,
        date: class.date,
        start_time: class.start_time,
        end_time: class.end_time,
        capacity: class.capacity,
        coach_id: class.coach_id,
        workout_id: class.workout_id,
        created_at: class.created_at,
        members,
    }))
}

#[derive(Deserialize)]
struct CreateClassBody {
    date: NaiveDate,
    start_time: chrono::NaiveTime,
    end_time: chrono::NaiveTime,
    #[serde(default = "default_capacity")]
    capacity: i32,
    coach_id: Option<String>,
    workout_id: Option<Uuid>,
}

fn default_capacity() -> i32 {
    20
}

async fn create_class(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(body): Json<CreateClassBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let class = crate::db::classes::create(
        &state.db,
        body.date,
        body.start_time,
        body.end_time,
        body.capacity,
        body.coach_id.as_deref(),
        body.workout_id,
    )
    .await?;
    Ok(Json(serde_json::json!(class)))
}

async fn signup_for_class(
    user: AuthUser,
    State(state): State<AppState>,
    Path(class_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !user.is_active {
        return Err(AppError::PaymentRequired);
    }

    let class = crate::db::classes::find_by_id(&state.db, class_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let count = crate::db::classes::signup_count(&state.db, class_id).await?;
    if count >= class.capacity as i64 {
        return Err(AppError::Conflict("Class is full".into()));
    }

    let signup = crate::db::classes::signup(&state.db, class_id, &user.id).await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e {
                if db_err.constraint() == Some("class_signups_class_id_user_id_key") {
                    return AppError::Conflict("Already signed up for this class".into());
                }
            }
            AppError::Database(e)
        })?;
    Ok(Json(serde_json::json!(signup)))
}

async fn cancel_signup(
    user: AuthUser,
    State(state): State<AppState>,
    Path(class_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let deleted = crate::db::classes::cancel_signup(&state.db, class_id, &user.id).await?;
    if !deleted {
        return Err(AppError::NotFound);
    }
    Ok(Json(serde_json::json!({ "success": true })))
}

#[derive(Deserialize)]
struct AttendanceBody {
    user_ids: Vec<String>,
}

async fn mark_attendance(
    _staff: StaffUser,
    State(state): State<AppState>,
    Path(class_id): Path<Uuid>,
    Json(body): Json<AttendanceBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Verify class exists
    crate::db::classes::find_by_id(&state.db, class_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let updated = crate::db::classes::mark_attendance(&state.db, class_id, &body.user_ids).await?;
    Ok(Json(serde_json::json!({ "updated": updated })))
}
