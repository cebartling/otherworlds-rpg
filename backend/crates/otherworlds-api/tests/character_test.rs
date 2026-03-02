//! Integration tests for the Character Management bounded context.

mod common;

use axum::http::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

/// Extract the `aggregate_id` from the `domain_events` table using a known `event_id`.
/// Needed because the character create handler generates the `character_id`
/// internally and only returns `event_ids`.
async fn aggregate_id_from_event(pool: &PgPool, event_id: Uuid) -> Uuid {
    let row: (Uuid,) = sqlx::query_as("SELECT aggregate_id FROM domain_events WHERE event_id = $1")
        .bind(event_id)
        .fetch_one(pool)
        .await
        .unwrap();
    row.0
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_character_create_round_trip(pool: PgPool) {
    let app = common::build_test_app(pool.clone());

    // POST /api/v1/characters/create
    let (status, json) = common::post_json(
        app,
        "/api/v1/characters/create",
        &serde_json::json!({ "name": "Alaric" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let event_ids = json["event_ids"].as_array().unwrap();
    assert_eq!(event_ids.len(), 1);

    // Extract the character_id from the stored event
    let event_id: Uuid = event_ids[0].as_str().unwrap().parse().unwrap();
    let character_id = aggregate_id_from_event(&pool, event_id).await;

    // GET /api/v1/characters/{character_id} — verify persisted state
    let app = common::build_test_app(pool);
    let (status, json) = common::get_json(app, &format!("/api/v1/characters/{character_id}")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["character_id"], character_id.to_string());
    assert_eq!(json["name"], "Alaric");
    assert_eq!(json["version"], 1);
    assert_eq!(json["experience"], 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_character_create_and_modify_attribute(pool: PgPool) {
    // Step 1: create character
    let app = common::build_test_app(pool.clone());
    let (status, json) = common::post_json(
        app,
        "/api/v1/characters/create",
        &serde_json::json!({ "name": "Brynna" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let event_id: Uuid = json["event_ids"][0].as_str().unwrap().parse().unwrap();
    let character_id = aggregate_id_from_event(&pool, event_id).await;

    // Step 2: modify attribute
    let app = common::build_test_app(pool.clone());
    let (status, json) = common::post_json(
        app,
        "/api/v1/characters/modify-attribute",
        &serde_json::json!({
            "character_id": character_id,
            "attribute": "strength",
            "new_value": 18
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["event_ids"].as_array().unwrap().len(), 1);

    // GET — verify both events applied
    let app = common::build_test_app(pool);
    let (status, json) = common::get_json(app, &format!("/api/v1/characters/{character_id}")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["version"], 2);
    assert_eq!(json["attributes"]["strength"], 18);
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_character_get_nonexistent_returns_404(pool: PgPool) {
    let app = common::build_test_app(pool);
    let character_id = Uuid::new_v4();

    let (status, json) = common::get_json(app, &format!("/api/v1/characters/{character_id}")).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(json["error"], "aggregate_not_found");
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_character_list_includes_created_character(pool: PgPool) {
    // Create a character
    let app = common::build_test_app(pool.clone());
    let (status, json) = common::post_json(
        app,
        "/api/v1/characters/create",
        &serde_json::json!({ "name": "Alaric" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let event_id: Uuid = json["event_ids"][0].as_str().unwrap().parse().unwrap();
    let character_id = aggregate_id_from_event(&pool, event_id).await;

    // GET /api/v1/characters — list should include the character
    let app = common::build_test_app(pool);
    let (status, json) = common::get_json(app, "/api/v1/characters").await;

    assert_eq!(status, StatusCode::OK);
    let characters = json.as_array().unwrap();
    assert!(
        characters
            .iter()
            .any(|c| c["character_id"] == character_id.to_string())
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_character_archive_round_trip(pool: PgPool) {
    let app = common::build_test_app(pool.clone());
    let (status, json) = common::post_json(
        app,
        "/api/v1/characters/create",
        &serde_json::json!({ "name": "Doomed" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let event_id: Uuid = json["event_ids"][0].as_str().unwrap().parse().unwrap();
    let character_id = aggregate_id_from_event(&pool, event_id).await;

    let app = common::build_test_app(pool.clone());
    let (status, json) = common::delete_json(
        app,
        &format!("/api/v1/characters/{character_id}"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["event_ids"].as_array().unwrap().len(), 1);

    let app = common::build_test_app(pool);
    let (status, json) = common::get_json(app, &format!("/api/v1/characters/{character_id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["version"], 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_character_archive_excludes_from_list(pool: PgPool) {
    let app = common::build_test_app(pool.clone());
    let (status, json) = common::post_json(
        app,
        "/api/v1/characters/create",
        &serde_json::json!({ "name": "Alice" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let event_id: Uuid = json["event_ids"][0].as_str().unwrap().parse().unwrap();
    let character_a = aggregate_id_from_event(&pool, event_id).await;

    let app = common::build_test_app(pool.clone());
    let (status, json) = common::post_json(
        app,
        "/api/v1/characters/create",
        &serde_json::json!({ "name": "Bob" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let event_id: Uuid = json["event_ids"][0].as_str().unwrap().parse().unwrap();
    let character_b = aggregate_id_from_event(&pool, event_id).await;

    let app = common::build_test_app(pool.clone());
    let (status, _json) = common::delete_json(
        app,
        &format!("/api/v1/characters/{character_a}"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let app = common::build_test_app(pool);
    let (status, json) = common::get_json(app, "/api/v1/characters").await;
    assert_eq!(status, StatusCode::OK);
    let characters = json.as_array().unwrap();
    assert!(
        !characters
            .iter()
            .any(|c| c["character_id"] == character_a.to_string())
    );
    assert!(
        characters
            .iter()
            .any(|c| c["character_id"] == character_b.to_string())
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_character_award_experience_round_trip(pool: PgPool) {
    let app = common::build_test_app(pool.clone());
    let (status, json) = common::post_json(
        app,
        "/api/v1/characters/create",
        &serde_json::json!({ "name": "Learner" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let event_id: Uuid = json["event_ids"][0].as_str().unwrap().parse().unwrap();
    let character_id = aggregate_id_from_event(&pool, event_id).await;

    let app = common::build_test_app(pool.clone());
    let (status, _json) = common::post_json(
        app,
        "/api/v1/characters/award-experience",
        &serde_json::json!({
            "character_id": character_id,
            "amount": 250
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let app = common::build_test_app(pool);
    let (status, json) = common::get_json(app, &format!("/api/v1/characters/{character_id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["experience"], 250);
    assert_eq!(json["version"], 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_character_command_on_archived_returns_error(pool: PgPool) {
    let app = common::build_test_app(pool.clone());
    let (status, json) = common::post_json(
        app,
        "/api/v1/characters/create",
        &serde_json::json!({ "name": "Doomed" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let event_id: Uuid = json["event_ids"][0].as_str().unwrap().parse().unwrap();
    let character_id = aggregate_id_from_event(&pool, event_id).await;

    let app = common::build_test_app(pool.clone());
    let (status, _json) = common::delete_json(
        app,
        &format!("/api/v1/characters/{character_id}"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let app = common::build_test_app(pool);
    let (status, json) = common::post_json(
        app,
        "/api/v1/characters/modify-attribute",
        &serde_json::json!({
            "character_id": character_id,
            "attribute": "strength",
            "new_value": 10
        }),
    )
    .await;
    assert_ne!(status, StatusCode::OK);
    assert!(json.get("error").is_some());
}
