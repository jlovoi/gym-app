mod admin;
mod health;
mod users;

use axum::Router;

use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .merge(health::router())
        .merge(crate::auth::router())
        .merge(users::router())
        .merge(admin::router())
        .with_state(state)
}
