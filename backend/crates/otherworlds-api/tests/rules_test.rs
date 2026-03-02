//! Integration tests for the Rules & Resolution bounded context.

mod common;

use axum::http::StatusCode;
use otherworlds_test_support::SequenceRng;
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test(migrations = "../../migrations")]
async fn test_rules_declare_intent_round_trip(pool: PgPool) {
    let app = common::build_test_app(pool.clone());
    let resolution_id = Uuid::new_v4();
    let intent_id = Uuid::new_v4();

    // POST /api/v1/rules/declare-intent
    let (status, json) = common::post_json(
        app,
        "/api/v1/rules/declare-intent",
        &serde_json::json!({
            "resolution_id": resolution_id,
            "intent_id": intent_id,
            "action_type": "skill_check",
            "difficulty_class": 15,
            "modifier": 3
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let event_ids = json["event_ids"].as_array().unwrap();
    assert_eq!(event_ids.len(), 1);

    // GET /api/v1/rules/{resolution_id} — verify persisted state
    let app = common::build_test_app(pool);
    let (status, json) = common::get_json(app, &format!("/api/v1/rules/{resolution_id}")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["resolution_id"], resolution_id.to_string());
    assert_eq!(json["phase"], "intent_declared");
    assert!(json["intent"].is_object());
    assert_eq!(json["version"], 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_rules_full_resolution_lifecycle(pool: PgPool) {
    let resolution_id = Uuid::new_v4();
    let intent_id = Uuid::new_v4();

    // Step 1: declare-intent
    let app = common::build_test_app(pool.clone());
    let (status, _) = common::post_json(
        app,
        "/api/v1/rules/declare-intent",
        &serde_json::json!({
            "resolution_id": resolution_id,
            "intent_id": intent_id,
            "action_type": "skill_check",
            "skill": "perception",
            "difficulty_class": 15,
            "modifier": 3
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Step 2: resolve-check (needs RNG for d20 roll + check_id + event_id)
    // SequenceRng values: d20 roll (15), four u32s for check_id, four u32s for event_id
    let rng = SequenceRng::new(vec![15, 42, 99, 7, 13, 1, 2, 3, 4]);
    let app = common::build_test_app_with_rng(pool.clone(), rng);
    let (status, json) = common::post_json(
        app,
        "/api/v1/rules/resolve-check",
        &serde_json::json!({ "resolution_id": resolution_id }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["event_ids"].as_array().unwrap().len(), 1);

    // Step 3: produce-effects
    let app = common::build_test_app(pool.clone());
    let (status, json) = common::post_json(
        app,
        "/api/v1/rules/produce-effects",
        &serde_json::json!({
            "resolution_id": resolution_id,
            "effects": [{
                "effect_type": "damage",
                "payload": { "amount": 8 }
            }]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["event_ids"].as_array().unwrap().len(), 1);

    // GET — verify final state
    let app = common::build_test_app(pool);
    let (status, json) = common::get_json(app, &format!("/api/v1/rules/{resolution_id}")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["phase"], "effects_produced");
    assert_eq!(json["version"], 3);
    assert!(json["intent"].is_object());
    assert!(json["check_result"].is_object());
    assert!(!json["effects"].as_array().unwrap().is_empty());
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_rules_resolve_check_without_intent_returns_400(pool: PgPool) {
    let app = common::build_test_app(pool);
    let resolution_id = Uuid::new_v4();

    // Attempt resolve-check on a fresh (nonexistent) resolution — should fail
    // because no intent has been declared.
    let (status, json) = common::post_json(
        app,
        "/api/v1/rules/resolve-check",
        &serde_json::json!({ "resolution_id": resolution_id }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(json["error"], "validation_error");
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_rules_get_nonexistent_resolution_returns_404(pool: PgPool) {
    let app = common::build_test_app(pool);
    let resolution_id = Uuid::new_v4();

    let (status, json) = common::get_json(app, &format!("/api/v1/rules/{resolution_id}")).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(json["error"], "aggregate_not_found");
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_rules_list_resolutions_includes_created_resolution(pool: PgPool) {
    let resolution_id = Uuid::new_v4();
    let intent_id = Uuid::new_v4();

    // Create a resolution via declare-intent
    let app = common::build_test_app(pool.clone());
    let (status, _) = common::post_json(
        app,
        "/api/v1/rules/declare-intent",
        &serde_json::json!({
            "resolution_id": resolution_id,
            "intent_id": intent_id,
            "action_type": "skill_check",
            "difficulty_class": 15,
            "modifier": 3
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // GET /api/v1/rules — list should include the resolution
    let app = common::build_test_app(pool);
    let (status, json) = common::get_json(app, "/api/v1/rules").await;

    assert_eq!(status, StatusCode::OK);
    let resolutions = json.as_array().unwrap();
    assert!(
        resolutions
            .iter()
            .any(|r| r["resolution_id"] == resolution_id.to_string())
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_rules_archive_resolution_round_trip(pool: PgPool) {
    let resolution_id = Uuid::new_v4();
    let intent_id = Uuid::new_v4();

    // Step 1: declare-intent
    let app = common::build_test_app(pool.clone());
    let (status, _) = common::post_json(
        app,
        "/api/v1/rules/declare-intent",
        &serde_json::json!({
            "resolution_id": resolution_id,
            "intent_id": intent_id,
            "action_type": "skill_check",
            "difficulty_class": 15,
            "modifier": 3
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Step 2: DELETE /api/v1/rules/{resolution_id}
    let app = common::build_test_app(pool.clone());
    let (status, json) = common::delete_json(app, &format!("/api/v1/rules/{resolution_id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["event_ids"].as_array().unwrap().len(), 1);

    // Step 3: GET — verify version incremented
    let app = common::build_test_app(pool);
    let (status, json) = common::get_json(app, &format!("/api/v1/rules/{resolution_id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["version"], 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_rules_archive_excludes_from_list(pool: PgPool) {
    let resolution_id_a = Uuid::new_v4();
    let intent_id_a = Uuid::new_v4();
    let resolution_id_b = Uuid::new_v4();
    let intent_id_b = Uuid::new_v4();

    // Create resolution_a
    let app = common::build_test_app(pool.clone());
    let (status, _) = common::post_json(
        app,
        "/api/v1/rules/declare-intent",
        &serde_json::json!({
            "resolution_id": resolution_id_a,
            "intent_id": intent_id_a,
            "action_type": "skill_check",
            "difficulty_class": 15,
            "modifier": 3
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Create resolution_b
    let app = common::build_test_app(pool.clone());
    let (status, _) = common::post_json(
        app,
        "/api/v1/rules/declare-intent",
        &serde_json::json!({
            "resolution_id": resolution_id_b,
            "intent_id": intent_id_b,
            "action_type": "skill_check",
            "difficulty_class": 15,
            "modifier": 3
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Archive resolution_a
    let app = common::build_test_app(pool.clone());
    let (status, _) = common::delete_json(app, &format!("/api/v1/rules/{resolution_id_a}")).await;
    assert_eq!(status, StatusCode::OK);

    // GET /api/v1/rules — resolution_a should NOT be in list, resolution_b should be
    let app = common::build_test_app(pool);
    let (status, json) = common::get_json(app, "/api/v1/rules").await;
    assert_eq!(status, StatusCode::OK);
    let resolutions = json.as_array().unwrap();
    assert!(
        !resolutions
            .iter()
            .any(|r| r["resolution_id"] == resolution_id_a.to_string())
    );
    assert!(
        resolutions
            .iter()
            .any(|r| r["resolution_id"] == resolution_id_b.to_string())
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_rules_command_on_archived_returns_error(pool: PgPool) {
    let resolution_id = Uuid::new_v4();
    let intent_id = Uuid::new_v4();

    // Step 1: declare-intent
    let app = common::build_test_app(pool.clone());
    let (status, _) = common::post_json(
        app,
        "/api/v1/rules/declare-intent",
        &serde_json::json!({
            "resolution_id": resolution_id,
            "intent_id": intent_id,
            "action_type": "skill_check",
            "difficulty_class": 15,
            "modifier": 3
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Step 2: archive the resolution
    let app = common::build_test_app(pool.clone());
    let (status, _) = common::delete_json(app, &format!("/api/v1/rules/{resolution_id}")).await;
    assert_eq!(status, StatusCode::OK);

    // Step 3: attempt resolve-check on archived resolution — should fail
    let rng = SequenceRng::new(vec![15, 42, 99, 7, 13]);
    let app = common::build_test_app_with_rng(pool, rng);
    let (status, json) = common::post_json(
        app,
        "/api/v1/rules/resolve-check",
        &serde_json::json!({ "resolution_id": resolution_id }),
    )
    .await;
    assert_ne!(status, StatusCode::OK);
    assert!(json.get("error").is_some());
}
