use std::{net::SocketAddr, sync::Arc};

use gym_app_backend::{app, config::AppConfig, db, jwt::Jwt, state::AppState};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "gym_app_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::from_env();

    let db = db::connect(&config.database_url)
        .await
        .expect("failed to connect to database / run migrations");

    let jwt = Jwt::new(
        &config.jwt_secret,
        config.access_token_ttl_secs,
        config.refresh_token_ttl_secs,
    );

    let http = reqwest::Client::builder()
        // Following redirects on OAuth token/userinfo requests opens the door to SSRF.
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("failed to build http client");

    let state = AppState {
        db,
        jwt,
        http,
        config: Arc::new(config),
    };

    let app = app(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
