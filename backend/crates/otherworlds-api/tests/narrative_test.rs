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

#[sqlx::test(migrations = "../../migrations")]
async fn test_narrative_list_sessions_includes_created_session(pool: PgPool) {
    let session_id = Uuid::new_v4();

    // Create a session via advance-beat
    let app = common::build_test_app(pool.clone());
    let (status, _) = common::post_json(
        app,
        "/api/v1/narrative/advance-beat",
        &serde_json::json!({ "session_id": session_id }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // GET /api/v1/narrative — list should include the session
    let app = common::build_test_app(pool);
    let (status, json) = common::get_json(app, "/api/v1/narrative").await;

    assert_eq!(status, StatusCode::OK);
    let sessions = json.as_array().unwrap();
    assert!(
        sessions
            .iter()
            .any(|s| s["session_id"] == session_id.to_string())
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_narrative_archive_session_round_trip(pool: PgPool) {
    let session_id = Uuid::new_v4();

    // POST /api/v1/narrative/advance-beat
    let app = common::build_test_app(pool.clone());
    let (status, _) = common::post_json(
        app,
        "/api/v1/narrative/advance-beat",
        &serde_json::json!({ "session_id": session_id }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // DELETE /api/v1/narrative/{session_id}
    let app = common::build_test_app(pool.clone());
    let (status, json) = common::delete_json(app, &format!("/api/v1/narrative/{session_id}")).await;
    assert_eq!(status, StatusCode::OK);
    let event_ids = json["event_ids"].as_array().unwrap();
    assert_eq!(event_ids.len(), 1);

    // GET /api/v1/narrative/{session_id} — verify version == 2
    let app = common::build_test_app(pool);
    let (status, json) = common::get_json(app, &format!("/api/v1/narrative/{session_id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["version"], 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_narrative_archive_excludes_from_list(pool: PgPool) {
    let session_a = Uuid::new_v4();
    let session_b = Uuid::new_v4();

    // Create session_a
    let app = common::build_test_app(pool.clone());
    let (status, _) = common::post_json(
        app,
        "/api/v1/narrative/advance-beat",
        &serde_json::json!({ "session_id": session_a }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Create session_b
    let app = common::build_test_app(pool.clone());
    let (status, _) = common::post_json(
        app,
        "/api/v1/narrative/advance-beat",
        &serde_json::json!({ "session_id": session_b }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Archive session_a
    let app = common::build_test_app(pool.clone());
    let (status, _) = common::delete_json(app, &format!("/api/v1/narrative/{session_a}")).await;
    assert_eq!(status, StatusCode::OK);

    // GET /api/v1/narrative — session_a should NOT be in list, session_b should
    let app = common::build_test_app(pool);
    let (status, json) = common::get_json(app, "/api/v1/narrative").await;
    assert_eq!(status, StatusCode::OK);
    let sessions = json.as_array().unwrap();
    assert!(
        !sessions
            .iter()
            .any(|s| s["session_id"] == session_a.to_string())
    );
    assert!(
        sessions
            .iter()
            .any(|s| s["session_id"] == session_b.to_string())
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_narrative_enter_scene_round_trip(pool: PgPool) {
    let app = common::build_test_app(pool.clone());
    let session_id = Uuid::new_v4();

    // POST /api/v1/narrative/enter-scene
    let (status, json) = common::post_json(
        app,
        "/api/v1/narrative/enter-scene",
        &serde_json::json!({
            "session_id": session_id,
            "scene_id": "tavern",
            "narrative_text": "You enter the tavern.",
            "choices": [
                { "label": "Leave", "target_scene_id": "street" },
                { "label": "Talk to barkeep", "target_scene_id": "barkeep_dialogue" }
            ],
            "npc_refs": ["barkeep"]
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let event_ids = json["event_ids"].as_array().unwrap();
    assert_eq!(event_ids.len(), 1);

    // GET /api/v1/narrative/{session_id} — verify scene state
    let app = common::build_test_app(pool);
    let (status, json) = common::get_json(app, &format!("/api/v1/narrative/{session_id}")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["session_id"], session_id.to_string());
    assert_eq!(json["current_scene_id"], "tavern");
    assert_eq!(json["scene_history"], serde_json::json!(["tavern"]));
    assert_eq!(json["active_choice_options"].as_array().unwrap().len(), 2);
    assert_eq!(json["version"], 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_narrative_select_choice_round_trip(pool: PgPool) {
    let session_id = Uuid::new_v4();

    // Enter scene A with 2 choices
    let app = common::build_test_app(pool.clone());
    let (status, _) = common::post_json(
        app,
        "/api/v1/narrative/enter-scene",
        &serde_json::json!({
            "session_id": session_id,
            "scene_id": "start",
            "narrative_text": "You stand at the crossroads.",
            "choices": [
                { "label": "Go north", "target_scene_id": "forest" },
                { "label": "Go south", "target_scene_id": "village" }
            ]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Select choice 0 → transitions to forest
    let app = common::build_test_app(pool.clone());
    let (status, json) = common::post_json(
        app,
        "/api/v1/narrative/select-choice",
        &serde_json::json!({
            "session_id": session_id,
            "choice_index": 0,
            "target_scene": {
                "scene_id": "forest",
                "narrative_text": "You enter the dark forest.",
                "choices": [
                    { "label": "Return", "target_scene_id": "start" }
                ]
            }
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let event_ids = json["event_ids"].as_array().unwrap();
    assert_eq!(event_ids.len(), 2); // ChoiceSelected + SceneStarted

    // GET — verify session transitioned to forest
    let app = common::build_test_app(pool);
    let (status, json) = common::get_json(app, &format!("/api/v1/narrative/{session_id}")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["current_scene_id"], "forest");
    assert_eq!(
        json["scene_history"],
        serde_json::json!(["start", "forest"])
    );
    assert_eq!(json["active_choice_options"].as_array().unwrap().len(), 1);
    assert_eq!(json["version"], 3); // enter_scene(1) + choice_selected(2) + scene_started(3)
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_narrative_full_gameplay_loop(pool: PgPool) {
    let session_id = Uuid::new_v4();

    // Enter scene A
    let app = common::build_test_app(pool.clone());
    let (status, _) = common::post_json(
        app,
        "/api/v1/narrative/enter-scene",
        &serde_json::json!({
            "session_id": session_id,
            "scene_id": "A",
            "narrative_text": "Scene A.",
            "choices": [{ "label": "Go to B", "target_scene_id": "B" }]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Select → B
    let app = common::build_test_app(pool.clone());
    let (status, _) = common::post_json(
        app,
        "/api/v1/narrative/select-choice",
        &serde_json::json!({
            "session_id": session_id,
            "choice_index": 0,
            "target_scene": {
                "scene_id": "B",
                "narrative_text": "Scene B.",
                "choices": [{ "label": "Go to C", "target_scene_id": "C" }]
            }
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Select → C
    let app = common::build_test_app(pool.clone());
    let (status, _) = common::post_json(
        app,
        "/api/v1/narrative/select-choice",
        &serde_json::json!({
            "session_id": session_id,
            "choice_index": 0,
            "target_scene": {
                "scene_id": "C",
                "narrative_text": "Scene C.",
                "choices": []
            }
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // GET — verify full history
    let app = common::build_test_app(pool);
    let (status, json) = common::get_json(app, &format!("/api/v1/narrative/{session_id}")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["current_scene_id"], "C");
    assert_eq!(json["scene_history"], serde_json::json!(["A", "B", "C"]));
    assert!(json["active_choice_options"].as_array().unwrap().is_empty());
    assert_eq!(json["version"], 5); // enter(1) + choice+enter(3) + choice+enter(5)
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_narrative_command_on_archived_returns_error(pool: PgPool) {
    let session_id = Uuid::new_v4();

    // Create session via advance-beat
    let app = common::build_test_app(pool.clone());
    let (status, _) = common::post_json(
        app,
        "/api/v1/narrative/advance-beat",
        &serde_json::json!({ "session_id": session_id }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Archive the session
    let app = common::build_test_app(pool.clone());
    let (status, _) = common::delete_json(app, &format!("/api/v1/narrative/{session_id}")).await;
    assert_eq!(status, StatusCode::OK);

    // POST advance-beat on archived session — should fail
    let app = common::build_test_app(pool);
    let (status, json) = common::post_json(
        app,
        "/api/v1/narrative/advance-beat",
        &serde_json::json!({ "session_id": session_id }),
    )
    .await;
    assert_ne!(status, StatusCode::OK);
    assert!(json["error"].is_string());
}
