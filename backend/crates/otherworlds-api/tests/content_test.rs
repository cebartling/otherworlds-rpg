//! Integration tests for the Content Authoring bounded context.

mod common;

use axum::http::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test(migrations = "../../migrations")]
async fn test_content_ingest_campaign_round_trip(pool: PgPool) {
    let app = common::build_test_app(pool.clone());

    // POST /api/v1/content/ingest-campaign
    let (status, json) = common::post_json(
        app,
        "/api/v1/content/ingest-campaign",
        &serde_json::json!({
            "source": "# My Campaign\n\nContent here."
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let campaign_id: Uuid = json["aggregate_id"].as_str().unwrap().parse().unwrap();
    assert_eq!(json["event_ids"].as_array().unwrap().len(), 1);

    // GET /api/v1/content/{campaign_id} — verify persisted state
    let app = common::build_test_app(pool);
    let (status, json) = common::get_json(app, &format!("/api/v1/content/{campaign_id}")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["campaign_id"], campaign_id.to_string());
    assert_eq!(json["ingested"], true);
    assert_eq!(json["validated"], false);
    assert_eq!(json["version"], 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_content_ingest_and_validate_campaign(pool: PgPool) {
    // Step 1: ingest
    let app = common::build_test_app(pool.clone());
    let (status, json) = common::post_json(
        app,
        "/api/v1/content/ingest-campaign",
        &serde_json::json!({
            "source": "---\ntitle: \"Campaign Two\"\n---\n\n# Scene: start\n\nAnother campaign.\n"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let campaign_id: Uuid = json["aggregate_id"].as_str().unwrap().parse().unwrap();

    // Step 2: validate
    let app = common::build_test_app(pool.clone());
    let (status, json) = common::post_json(
        app,
        "/api/v1/content/validate-campaign",
        &serde_json::json!({ "campaign_id": campaign_id }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["event_ids"].as_array().unwrap().len(), 1);

    // GET — verify both flags
    let app = common::build_test_app(pool);
    let (status, json) = common::get_json(app, &format!("/api/v1/content/{campaign_id}")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["ingested"], true);
    assert_eq!(json["validated"], true);
    assert_eq!(json["version"], 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_content_get_nonexistent_returns_404(pool: PgPool) {
    let app = common::build_test_app(pool);
    let campaign_id = Uuid::new_v4();

    let (status, json) = common::get_json(app, &format!("/api/v1/content/{campaign_id}")).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(json["error"], "aggregate_not_found");
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_content_list_includes_ingested_campaign(pool: PgPool) {
    // Ingest a campaign
    let app = common::build_test_app(pool.clone());
    let (status, json) = common::post_json(
        app,
        "/api/v1/content/ingest-campaign",
        &serde_json::json!({
            "source": "# My Campaign\n\nContent here."
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let campaign_id: Uuid = json["aggregate_id"].as_str().unwrap().parse().unwrap();

    // GET /api/v1/content — list should include the campaign
    let app = common::build_test_app(pool);
    let (status, json) = common::get_json(app, "/api/v1/content").await;

    assert_eq!(status, StatusCode::OK);
    let campaigns = json.as_array().unwrap();
    assert!(
        campaigns
            .iter()
            .any(|c| c["campaign_id"] == campaign_id.to_string())
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_content_archive_campaign_round_trip(pool: PgPool) {
    let app = common::build_test_app(pool.clone());
    let (status, json) = common::post_json(
        app,
        "/api/v1/content/ingest-campaign",
        &serde_json::json!({
            "source": "# Archive Test\n\nContent."
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let campaign_id: Uuid = json["aggregate_id"].as_str().unwrap().parse().unwrap();

    let app = common::build_test_app(pool.clone());
    let (status, json) = common::delete_json(app, &format!("/api/v1/content/{campaign_id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["event_ids"].as_array().unwrap().len(), 1);

    let app = common::build_test_app(pool);
    let (status, json) = common::get_json(app, &format!("/api/v1/content/{campaign_id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["version"], 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_content_archive_excludes_from_list(pool: PgPool) {
    let app = common::build_test_app(pool.clone());
    let (status, json) = common::post_json(
        app,
        "/api/v1/content/ingest-campaign",
        &serde_json::json!({
            "source": "# Campaign A\n\nFirst campaign."
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let campaign_a: Uuid = json["aggregate_id"].as_str().unwrap().parse().unwrap();

    let app = common::build_test_app(pool.clone());
    let (status, json) = common::post_json(
        app,
        "/api/v1/content/ingest-campaign",
        &serde_json::json!({
            "source": "# Campaign B\n\nSecond campaign."
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let campaign_b: Uuid = json["aggregate_id"].as_str().unwrap().parse().unwrap();

    let app = common::build_test_app(pool.clone());
    let (status, _json) = common::delete_json(app, &format!("/api/v1/content/{campaign_a}")).await;
    assert_eq!(status, StatusCode::OK);

    let app = common::build_test_app(pool);
    let (status, json) = common::get_json(app, "/api/v1/content").await;
    assert_eq!(status, StatusCode::OK);
    let campaigns = json.as_array().unwrap();
    assert!(
        !campaigns
            .iter()
            .any(|c| c["campaign_id"] == campaign_a.to_string())
    );
    assert!(
        campaigns
            .iter()
            .any(|c| c["campaign_id"] == campaign_b.to_string())
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_content_full_lifecycle_ingest_validate_compile(pool: PgPool) {
    let app = common::build_test_app(pool.clone());
    let (status, json) = common::post_json(
        app,
        "/api/v1/content/ingest-campaign",
        &serde_json::json!({
            "source": "---\ntitle: \"Full Lifecycle\"\n---\n\n# Scene: start\n\nContent.\n"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let campaign_id: Uuid = json["aggregate_id"].as_str().unwrap().parse().unwrap();

    let app = common::build_test_app(pool.clone());
    let (status, _json) = common::post_json(
        app,
        "/api/v1/content/validate-campaign",
        &serde_json::json!({ "campaign_id": campaign_id }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let app = common::build_test_app(pool.clone());
    let (status, _json) = common::post_json(
        app,
        "/api/v1/content/compile-campaign",
        &serde_json::json!({ "campaign_id": campaign_id }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let app = common::build_test_app(pool);
    let (status, json) = common::get_json(app, &format!("/api/v1/content/{campaign_id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["version"], 3);
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_content_command_on_archived_returns_error(pool: PgPool) {
    let app = common::build_test_app(pool.clone());
    let (status, json) = common::post_json(
        app,
        "/api/v1/content/ingest-campaign",
        &serde_json::json!({
            "source": "# Archived Campaign\n\nContent."
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let campaign_id: Uuid = json["aggregate_id"].as_str().unwrap().parse().unwrap();

    let app = common::build_test_app(pool.clone());
    let (status, _json) = common::delete_json(app, &format!("/api/v1/content/{campaign_id}")).await;
    assert_eq!(status, StatusCode::OK);

    let app = common::build_test_app(pool);
    let (status, json) = common::post_json(
        app,
        "/api/v1/content/validate-campaign",
        &serde_json::json!({ "campaign_id": campaign_id }),
    )
    .await;
    assert_ne!(status, StatusCode::OK);
    assert!(json.get("error").is_some());
}
