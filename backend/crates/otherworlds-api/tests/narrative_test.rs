//! Integration tests for the Narrative bounded context.

mod common;

use axum::http::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test(migrations = "../../migrations")]
async fn test_narrative_advance_beat_round_trip(pool: PgPool) {
    let app = common::build_test_app(pool.clone());
    let session_id = Uuid::new_v4();

    // POST /api/v1/narrative/advance-beat
    let (status, json) = common::post_json(
        app,
        "/api/v1/narrative/advance-beat",
        &serde_json::json!({ "session_id": session_id }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let event_ids = json["event_ids"].as_array().unwrap();
    assert_eq!(event_ids.len(), 1);

    // GET /api/v1/narrative/{session_id} — verify persisted state
    let app = common::build_test_app(pool);
    let (status, json) = common::get_json(app, &format!("/api/v1/narrative/{session_id}")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["session_id"], session_id.to_string());
    assert_eq!(json["version"], 1);
    assert!(json["current_beat_id"].is_string());
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_narrative_present_choice_round_trip(pool: PgPool) {
    let app = common::build_test_app(pool.clone());
    let session_id = Uuid::new_v4();

    // POST /api/v1/narrative/present-choice
    let (status, json) = common::post_json(
        app,
        "/api/v1/narrative/present-choice",
        &serde_json::json!({ "session_id": session_id }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let event_ids = json["event_ids"].as_array().unwrap();
    assert_eq!(event_ids.len(), 1);

    // GET — verify persisted state
    let app = common::build_test_app(pool);
    let (status, json) = common::get_json(app, &format!("/api/v1/narrative/{session_id}")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["version"], 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_narrative_get_nonexistent_session_returns_404(pool: PgPool) {
    let app = common::build_test_app(pool);
    let session_id = Uuid::new_v4();

    let (status, json) = common::get_json(app, &format!("/api/v1/narrative/{session_id}")).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(json["error"], "aggregate_not_found");
}
