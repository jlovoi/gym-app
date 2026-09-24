use gym_backend::auth::middleware::JwksCache;
use gym_backend::config::Config;
use gym_backend::state::AppState;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let config = Config::from_env();

    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("Failed to run migrations");

    tracing::info!("Migrations applied successfully");

    let jwks = JwksCache::new(config.clerk_jwks_url.clone());
    jwks.refresh().await.expect("Failed to fetch JWKS keys");
    tracing::info!("JWKS keys loaded");

    let state = AppState {
        db,
        config: config.clone(),
        jwks,
    };

    let app = gym_backend::routes::router(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let addr = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app).await.expect("Server error");
}
