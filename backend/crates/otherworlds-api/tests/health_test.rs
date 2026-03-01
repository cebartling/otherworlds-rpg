//! Integration tests for the health endpoint.

mod common;

use axum::http::StatusCode;
use sqlx::PgPool;

#[sqlx::test(migrations = "../../migrations")]
async fn test_health_returns_200_with_status_ok(pool: PgPool) {
    let app = common::build_test_app(pool);

    let (status, json) = common::get_json(app, "/health").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["status"], "ok");
    assert!(json["version"].is_string());
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_unknown_route_returns_404(pool: PgPool) {
    let app = common::build_test_app(pool);

    let request = axum::http::Request::builder()
        .method("GET")
        .uri("/api/v1/nonexistent")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = tower::ServiceExt::oneshot(app, request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
