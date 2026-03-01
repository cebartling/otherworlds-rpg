//! Integration tests for the Session & Progress bounded context.

mod common;

use axum::http::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test(migrations = "../../migrations")]
async fn test_session_start_campaign_run_round_trip(pool: PgPool) {
    let app = common::build_test_app(pool.clone());
    let campaign_id = Uuid::new_v4();

    // POST /api/v1/sessions/start-campaign-run
    let (status, json) = common::post_json(
        app,
        "/api/v1/sessions/start-campaign-run",
        &serde_json::json!({ "campaign_id": campaign_id }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let run_id: Uuid = json["aggregate_id"].as_str().unwrap().parse().unwrap();
    assert_eq!(json["event_ids"].as_array().unwrap().len(), 1);

    // GET /api/v1/sessions/{run_id} — verify persisted state
    let app = common::build_test_app(pool);
    let (status, json) = common::get_json(app, &format!("/api/v1/sessions/{run_id}")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["run_id"], run_id.to_string());
    assert_eq!(json["campaign_id"], campaign_id.to_string());
    assert_eq!(json["version"], 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_session_create_checkpoint_round_trip(pool: PgPool) {
    let campaign_id = Uuid::new_v4();

    // Step 1: start campaign run
    let app = common::build_test_app(pool.clone());
    let (status, json) = common::post_json(
        app,
        "/api/v1/sessions/start-campaign-run",
        &serde_json::json!({ "campaign_id": campaign_id }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let run_id: Uuid = json["aggregate_id"].as_str().unwrap().parse().unwrap();

    // Step 2: create checkpoint
    let app = common::build_test_app(pool.clone());
    let (status, json) = common::post_json(
        app,
        "/api/v1/sessions/create-checkpoint",
        &serde_json::json!({ "run_id": run_id }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["aggregate_id"], run_id.to_string());
    assert_eq!(json["event_ids"].as_array().unwrap().len(), 1);

    // GET — verify version incremented
    let app = common::build_test_app(pool);
    let (status, json) = common::get_json(app, &format!("/api/v1/sessions/{run_id}")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["version"], 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_session_get_nonexistent_returns_404(pool: PgPool) {
    let app = common::build_test_app(pool);
    let run_id = Uuid::new_v4();

    let (status, json) = common::get_json(app, &format!("/api/v1/sessions/{run_id}")).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(json["error"], "aggregate_not_found");
}
