//! Integration tests for the World State bounded context.

mod common;

use axum::http::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test(migrations = "../../migrations")]
async fn test_world_apply_effect_round_trip(pool: PgPool) {
    let app = common::build_test_app(pool.clone());
    let world_id = Uuid::new_v4();

    // POST /api/v1/world/apply-effect
    let (status, json) = common::post_json(
        app,
        "/api/v1/world/apply-effect",
        &serde_json::json!({
            "world_id": world_id,
            "fact_key": "quest_complete"
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["event_ids"].as_array().unwrap().len(), 1);

    // GET /api/v1/world/{world_id} — verify persisted state
    let app = common::build_test_app(pool);
    let (status, json) = common::get_json(app, &format!("/api/v1/world/{world_id}")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["world_id"], world_id.to_string());
    assert_eq!(json["facts"], serde_json::json!(["quest_complete"]));
    assert_eq!(json["version"], 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_world_set_flag_round_trip(pool: PgPool) {
    let app = common::build_test_app(pool.clone());
    let world_id = Uuid::new_v4();

    // POST /api/v1/world/set-flag
    let (status, json) = common::post_json(
        app,
        "/api/v1/world/set-flag",
        &serde_json::json!({
            "world_id": world_id,
            "flag_key": "door_unlocked",
            "value": true
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["event_ids"].as_array().unwrap().len(), 1);

    // GET — verify persisted state
    let app = common::build_test_app(pool);
    let (status, json) = common::get_json(app, &format!("/api/v1/world/{world_id}")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["version"], 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_world_get_nonexistent_returns_404(pool: PgPool) {
    let app = common::build_test_app(pool);
    let world_id = Uuid::new_v4();

    let (status, json) = common::get_json(app, &format!("/api/v1/world/{world_id}")).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(json["error"], "aggregate_not_found");
}
