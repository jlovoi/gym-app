use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};

use crate::auth::middleware::AuthUser;
use crate::db::{memberships as memberships_db, users as users_db};
use crate::error::AppError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/users/me", get(get_me))
}

async fn get_me(
    user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    let db_user = users_db::find_by_id(&state.db, &user.id)
        .await?
        .ok_or(AppError::NotFound)?;

    let membership = memberships_db::find_active_by_user(&state.db, &user.id).await?;

    Ok(Json(json!({
        "user": db_user,
        "membership": membership,
    })))
}
