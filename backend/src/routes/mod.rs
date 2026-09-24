mod admin;
mod classes;
mod frontend;
mod health;
mod logs;
mod memberships;
mod users;
mod webhooks;
mod workouts;

use axum::Router;

use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .merge(frontend::router())
        .merge(health::router())
        .merge(workouts::router())
        .merge(logs::router())
        .merge(classes::router())
        .merge(memberships::router())
        .merge(users::router())
        .merge(admin::router())
        .merge(webhooks::router())
        .with_state(state)
}
