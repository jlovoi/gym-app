use axum::routing::get;
use axum::{Json, Router};

use crate::auth::extractors::CurrentUser;
use crate::models::user::User;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/users/me", get(get_me))
}

async fn get_me(CurrentUser(user): CurrentUser) -> Json<User> {
    Json(user)
}
