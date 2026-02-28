//! Otherworlds RPG API server entry point.

use std::net::SocketAddr;

use axum::Router;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

mod routes;
mod state;

#[tokio::main]
async fn main() {
    // Initialize tracing subscriber.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .json()
        .init();

    tracing::info!("Starting Otherworlds RPG API server");

    // Read configuration from environment.
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://otherworlds:otherworlds@localhost:5432/otherworlds".to_string()
    });
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a valid u16");

    // Create database connection pool.
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    // Build application state.
    let app_state = state::AppState::new(pool);

    // Build router.
    let app = Router::new()
        .merge(routes::health::router())
        .nest("/api/v1/narrative", routes::narrative::router())
        .nest("/api/v1/rules", routes::rules::router())
        .nest("/api/v1/world", routes::world_state::router())
        .nest("/api/v1/characters", routes::character::router())
        .nest("/api/v1/inventory", routes::inventory::router())
        .nest("/api/v1/sessions", routes::session::router())
        .nest("/api/v1/content", routes::content::router())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    // Start server.
    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .expect("Invalid HOST:PORT combination");
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}
