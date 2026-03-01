//! Integration tests for the Inventory & Economy bounded context.

mod common;

use axum::http::StatusCode;
use chrono::Utc;
use otherworlds_core::repository::{EventRepository, StoredEvent};
use otherworlds_event_store::pg_event_repository::PgEventRepository;
use otherworlds_inventory::domain::events::{ITEM_ADDED_EVENT_TYPE, InventoryEventKind, ItemAdded};
use sqlx::PgPool;
use uuid::Uuid;

/// Seed an inventory with a single item by writing an `ItemAdded` event directly.
async fn seed_inventory(pool: &PgPool, inventory_id: Uuid, item_id: Uuid) {
    let repo = PgEventRepository::new(pool.clone());
    let event = StoredEvent {
        event_id: Uuid::new_v4(),
        aggregate_id: inventory_id,
        event_type: ITEM_ADDED_EVENT_TYPE.to_owned(),
        payload: serde_json::to_value(InventoryEventKind::ItemAdded(ItemAdded {
            inventory_id,
            item_id,
        }))
        .unwrap(),
        sequence_number: 1,
        correlation_id: Uuid::new_v4(),
        causation_id: Uuid::new_v4(),
        occurred_at: Utc::now(),
    };
    repo.append_events(inventory_id, 0, &[event]).await.unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_inventory_add_item_round_trip(pool: PgPool) {
    let inventory_id = Uuid::new_v4();
    let seed_item_id = Uuid::new_v4();
    let new_item_id = Uuid::new_v4();

    // Seed inventory so it exists
    seed_inventory(&pool, inventory_id, seed_item_id).await;

    // POST /api/v1/inventory/add-item — add a second item
    let app = common::build_test_app(pool.clone());
    let (status, json) = common::post_json(
        app,
        "/api/v1/inventory/add-item",
        &serde_json::json!({
            "inventory_id": inventory_id,
            "item_id": new_item_id
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["aggregate_id"], inventory_id.to_string());
    assert_eq!(json["event_ids"].as_array().unwrap().len(), 1);

    // GET /api/v1/inventory/{inventory_id} — verify persisted state
    let app = common::build_test_app(pool);
    let (status, json) = common::get_json(app, &format!("/api/v1/inventory/{inventory_id}")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["inventory_id"], inventory_id.to_string());
    let items = json["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(json["version"], 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_inventory_add_and_remove_item(pool: PgPool) {
    let inventory_id = Uuid::new_v4();
    let item_id = Uuid::new_v4();

    // Seed inventory with the item we'll remove
    seed_inventory(&pool, inventory_id, item_id).await;

    // Remove the seeded item
    let app = common::build_test_app(pool.clone());
    let (status, json) = common::post_json(
        app,
        "/api/v1/inventory/remove-item",
        &serde_json::json!({
            "inventory_id": inventory_id,
            "item_id": item_id
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["event_ids"].as_array().unwrap().len(), 1);

    // GET — verify empty items
    let app = common::build_test_app(pool);
    let (status, json) = common::get_json(app, &format!("/api/v1/inventory/{inventory_id}")).await;

    assert_eq!(status, StatusCode::OK);
    let items = json["items"].as_array().unwrap();
    assert!(items.is_empty());
    assert_eq!(json["version"], 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_inventory_get_nonexistent_returns_404(pool: PgPool) {
    let app = common::build_test_app(pool);
    let inventory_id = Uuid::new_v4();

    let (status, json) = common::get_json(app, &format!("/api/v1/inventory/{inventory_id}")).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(json["error"], "aggregate_not_found");
}
