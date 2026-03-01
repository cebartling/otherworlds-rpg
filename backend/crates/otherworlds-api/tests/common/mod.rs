//! Shared test helpers for API integration tests.
#![allow(dead_code)]

use std::sync::{Arc, Mutex};

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use otherworlds_core::clock::Clock;
use otherworlds_core::rng::DeterministicRng;
use otherworlds_event_store::pg_event_repository::PgEventRepository;
use otherworlds_test_support::{FixedClock, SequenceRng};
use sqlx::PgPool;
use tower::ServiceExt;

use otherworlds_api::routes;
use otherworlds_api::state::AppState;

/// Fixed timestamp used across all integration tests.
fn fixed_clock() -> Arc<dyn Clock + Send + Sync> {
    Arc::new(FixedClock(
        chrono::TimeZone::with_ymd_and_hms(&chrono::Utc, 2026, 1, 15, 10, 0, 0).unwrap(),
    ))
}

/// Build the full app router with a real `PgEventRepository` and deterministic
/// Clock/RNG. Uses the same route structure as `main.rs`.
pub fn build_test_app(pool: PgPool) -> Router {
    build_test_app_with_rng(pool, SequenceRng::new(vec![]))
}

/// Build the full app router with a custom `SequenceRng` for tests that need
/// deterministic dice rolls (e.g., rules context).
pub fn build_test_app_with_rng(pool: PgPool, rng: SequenceRng) -> Router {
    let clock = fixed_clock();
    let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(rng));
    let event_repository = Arc::new(PgEventRepository::new(pool.clone()));
    let app_state = AppState::new(pool, clock, rng, event_repository);

    Router::new()
        .merge(routes::health::router())
        .nest("/api/v1/narrative", routes::narrative::router())
        .nest("/api/v1/rules", routes::rules::router())
        .nest("/api/v1/world", routes::world_state::router())
        .nest("/api/v1/characters", routes::character::router())
        .nest("/api/v1/inventory", routes::inventory::router())
        .nest("/api/v1/sessions", routes::session::router())
        .nest("/api/v1/content", routes::content::router())
        .with_state(app_state)
}

/// Send a POST request with a JSON body and return the response.
pub async fn post_json(
    app: Router,
    uri: &str,
    body: &serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let request = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    (status, json)
}

/// Send a GET request and return the response.
pub async fn get_json(app: Router, uri: &str) -> (StatusCode, serde_json::Value) {
    let request = Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    (status, json)
}
