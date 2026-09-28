// admin and users need an auth extractor; unmounted until Clerk's replacement lands.
// mod admin;
mod health;
// mod users;

use axum::Router;

use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .merge(health::router())
        .with_state(state)
}
