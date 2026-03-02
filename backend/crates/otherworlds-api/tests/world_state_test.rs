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

#[sqlx::test(migrations = "../../migrations")]
async fn test_world_list_snapshots_includes_created_world(pool: PgPool) {
    let world_id = Uuid::new_v4();

    // Create a world snapshot via apply-effect
    let app = common::build_test_app(pool.clone());
    let (status, _) = common::post_json(
        app,
        "/api/v1/world/apply-effect",
        &serde_json::json!({
            "world_id": world_id,
            "fact_key": "quest_complete"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // GET /api/v1/world — list should include the world snapshot
    let app = common::build_test_app(pool);
    let (status, json) = common::get_json(app, "/api/v1/world").await;

    assert_eq!(status, StatusCode::OK);
    let snapshots = json.as_array().unwrap();
    assert!(
        snapshots
            .iter()
            .any(|s| s["world_id"] == world_id.to_string())
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_world_archive_round_trip(pool: PgPool) {
    let world_id = Uuid::new_v4();

    let app = common::build_test_app(pool.clone());
    let (status, _) = common::post_json(
        app,
        "/api/v1/world/apply-effect",
        &serde_json::json!({
            "world_id": world_id,
            "fact_key": "quest_started"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let app = common::build_test_app(pool.clone());
    let (status, json) =
        common::delete_json(app, &format!("/api/v1/world/{world_id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["event_ids"].as_array().unwrap().len(), 1);

    let app = common::build_test_app(pool);
    let (status, json) = common::get_json(app, &format!("/api/v1/world/{world_id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["version"], 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_world_archive_excludes_from_list(pool: PgPool) {
    let world_a = Uuid::new_v4();
    let world_b = Uuid::new_v4();

    let app = common::build_test_app(pool.clone());
    let (status, _) = common::post_json(
        app,
        "/api/v1/world/apply-effect",
        &serde_json::json!({
            "world_id": world_a,
            "fact_key": "init_a"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let app = common::build_test_app(pool.clone());
    let (status, _) = common::post_json(
        app,
        "/api/v1/world/apply-effect",
        &serde_json::json!({
            "world_id": world_b,
            "fact_key": "init_b"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let app = common::build_test_app(pool.clone());
    let (status, _) =
        common::delete_json(app, &format!("/api/v1/world/{world_a}")).await;
    assert_eq!(status, StatusCode::OK);

    let app = common::build_test_app(pool);
    let (status, json) = common::get_json(app, "/api/v1/world").await;
    assert_eq!(status, StatusCode::OK);
    let snapshots = json.as_array().unwrap();
    assert!(
        !snapshots
            .iter()
            .any(|s| s["world_id"] == world_a.to_string())
    );
    assert!(
        snapshots
            .iter()
            .any(|s| s["world_id"] == world_b.to_string())
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_world_update_disposition_round_trip(pool: PgPool) {
    let world_id = Uuid::new_v4();
    let entity_id = Uuid::new_v4();

    let app = common::build_test_app(pool.clone());
    let (status, _) = common::post_json(
        app,
        "/api/v1/world/apply-effect",
        &serde_json::json!({
            "world_id": world_id,
            "fact_key": "init"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let app = common::build_test_app(pool.clone());
    let (status, _) = common::post_json(
        app,
        "/api/v1/world/update-disposition",
        &serde_json::json!({
            "world_id": world_id,
            "entity_id": entity_id
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let app = common::build_test_app(pool);
    let (status, json) = common::get_json(app, &format!("/api/v1/world/{world_id}")).await;
    assert_eq!(status, StatusCode::OK);
    let disposition_ids = json["disposition_entity_ids"].as_array().unwrap();
    assert!(
        disposition_ids
            .iter()
            .any(|id| id == &serde_json::json!(entity_id.to_string()))
    );
    assert_eq!(json["version"], 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_world_command_on_archived_returns_error(pool: PgPool) {
    let world_id = Uuid::new_v4();

    let app = common::build_test_app(pool.clone());
    let (status, _) = common::post_json(
        app,
        "/api/v1/world/apply-effect",
        &serde_json::json!({
            "world_id": world_id,
            "fact_key": "init"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let app = common::build_test_app(pool.clone());
    let (status, _) =
        common::delete_json(app, &format!("/api/v1/world/{world_id}")).await;
    assert_eq!(status, StatusCode::OK);

    let app = common::build_test_app(pool);
    let (status, json) = common::post_json(
        app,
        "/api/v1/world/apply-effect",
        &serde_json::json!({
            "world_id": world_id,
            "fact_key": "new_quest"
        }),
    )
    .await;
    assert_ne!(status, StatusCode::OK);
    assert!(json.get("error").is_some());
}
