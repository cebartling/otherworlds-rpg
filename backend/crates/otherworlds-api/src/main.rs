//! Otherworlds RPG — API server entry point.

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use axum::Router;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use opentelemetry::trace::TracerProvider;

use otherworlds_api::error::AppError;
use otherworlds_api::{routes, state};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Initialize tracing subscriber with JSON stdout and optional OTLP export.
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let fmt_layer = tracing_subscriber::fmt::layer().json();

    let registry = tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer);

    if std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").is_ok() {
        let tracer = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .build()
            .map_err(|e| AppError::Config(format!("failed to create OTLP exporter: {e}")))?;

        let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
            .with_batch_exporter(tracer)
            .with_resource(
                opentelemetry_sdk::Resource::builder()
                    .with_service_name("otherworlds-api")
                    .build(),
            )
            .build();

        let otel_layer =
            tracing_opentelemetry::layer().with_tracer(tracer_provider.tracer("otherworlds-api"));

        registry.with(otel_layer).init();
    } else {
        registry.init();
    }

    tracing::info!("Starting Otherworlds RPG API server");

    // Read configuration from environment.
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| AppError::Config("DATABASE_URL environment variable must be set".into()))?;
    let host = std::env::var("HOST")
        .map_err(|_| AppError::Config("HOST environment variable must be set".into()))?;
    let port: u16 = std::env::var("PORT")
        .map_err(|_| AppError::Config("PORT environment variable must be set".into()))?
        .parse()
        .map_err(|e| AppError::Config(format!("PORT must be a valid u16: {e}")))?;

    // Validate HOST:PORT combination early.
    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .map_err(|e| AppError::Config(format!("invalid HOST:PORT combination: {e}")))?;

    // Create database connection pool.
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    // Build application state with injected Clock, RNG, and EventRepository.
    let clock: Arc<dyn otherworlds_core::clock::Clock + Send + Sync> =
        Arc::new(otherworlds_core::clock::SystemClock);
    let rng: Arc<Mutex<dyn otherworlds_core::rng::DeterministicRng + Send>> =
        Arc::new(Mutex::new(otherworlds_core::rng::StdRng));
    let event_repository: Arc<dyn otherworlds_core::repository::EventRepository> = Arc::new(
        otherworlds_event_store::pg_event_repository::PgEventRepository::new(pool.clone()),
    );
    let app_state = state::AppState::new(pool, clock, rng, event_repository);

    // Build CORS layer.
    let cors = if std::env::var("CORS_PERMISSIVE").is_ok_and(|v| v == "true") {
        tracing::warn!("CORS_PERMISSIVE=true — using permissive CORS policy");
        CorsLayer::permissive()
    } else {
        tracing::info!("Using default restrictive CORS policy");
        CorsLayer::new()
    };

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
        .nest("/api/v1/play", routes::play::router())
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(app_state);

    // Start server.
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
