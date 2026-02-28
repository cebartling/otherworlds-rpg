//! Routes for the Inventory & Economy bounded context.

use axum::extract::State;
use axum::{Json, Router, routing::post};
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};
use uuid::Uuid;

use otherworlds_inventory::application::command_handlers;
use otherworlds_inventory::domain::commands;

use crate::error::ApiError;
use crate::state::AppState;

/// Request body for POST /add-item.
#[derive(Debug, Deserialize)]
pub struct AddItemRequest {
    /// The inventory to add the item to.
    pub inventory_id: Uuid,
    /// The item to add.
    pub item_id: Uuid,
}

/// Request body for POST /remove-item.
#[derive(Debug, Deserialize)]
pub struct RemoveItemRequest {
    /// The inventory to remove the item from.
    pub inventory_id: Uuid,
    /// The item to remove.
    pub item_id: Uuid,
}

/// Request body for POST /equip-item.
#[derive(Debug, Deserialize)]
pub struct EquipItemRequest {
    /// The inventory containing the item.
    pub inventory_id: Uuid,
    /// The item to equip.
    pub item_id: Uuid,
}

/// Response body returned after a command is successfully handled.
#[derive(Debug, Serialize)]
pub struct CommandResponse {
    /// The aggregate ID affected by the command.
    pub aggregate_id: Uuid,
    /// IDs of the domain events produced and persisted.
    pub event_ids: Vec<Uuid>,
}

/// POST /add-item
#[instrument(skip(state, request), fields(inventory_id = %request.inventory_id))]
async fn add_item(
    State(state): State<AppState>,
    Json(request): Json<AddItemRequest>,
) -> Result<Json<CommandResponse>, ApiError> {
    let command = commands::AddItem {
        correlation_id: Uuid::new_v4(),
        inventory_id: request.inventory_id,
        item_id: request.item_id,
    };

    info!(correlation_id = %command.correlation_id, "handling add_item command");

    let result =
        command_handlers::handle_add_item(&command, state.clock.as_ref(), &*state.event_repository)
            .await?;

    let event_ids = result.stored_events.iter().map(|e| e.event_id).collect();

    Ok(Json(CommandResponse {
        aggregate_id: result.aggregate_id,
        event_ids,
    }))
}

/// POST /remove-item
#[instrument(skip(state, request), fields(inventory_id = %request.inventory_id))]
async fn remove_item(
    State(state): State<AppState>,
    Json(request): Json<RemoveItemRequest>,
) -> Result<Json<CommandResponse>, ApiError> {
    let command = commands::RemoveItem {
        correlation_id: Uuid::new_v4(),
        inventory_id: request.inventory_id,
        item_id: request.item_id,
    };

    info!(correlation_id = %command.correlation_id, "handling remove_item command");

    let result = command_handlers::handle_remove_item(
        &command,
        state.clock.as_ref(),
        &*state.event_repository,
    )
    .await?;

    let event_ids = result.stored_events.iter().map(|e| e.event_id).collect();

    Ok(Json(CommandResponse {
        aggregate_id: result.aggregate_id,
        event_ids,
    }))
}

/// POST /equip-item
#[instrument(skip(state, request), fields(inventory_id = %request.inventory_id))]
async fn equip_item(
    State(state): State<AppState>,
    Json(request): Json<EquipItemRequest>,
) -> Result<Json<CommandResponse>, ApiError> {
    let command = commands::EquipItem {
        correlation_id: Uuid::new_v4(),
        inventory_id: request.inventory_id,
        item_id: request.item_id,
    };

    info!(correlation_id = %command.correlation_id, "handling equip_item command");

    let result = command_handlers::handle_equip_item(
        &command,
        state.clock.as_ref(),
        &*state.event_repository,
    )
    .await?;

    let event_ids = result.stored_events.iter().map(|e| e.event_id).collect();

    Ok(Json(CommandResponse {
        aggregate_id: result.aggregate_id,
        event_ids,
    }))
}

/// Returns the router for the inventory context.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/add-item", post(add_item))
        .route("/remove-item", post(remove_item))
        .route("/equip-item", post(equip_item))
}

#[cfg(test)]
mod tests {
    use super::*;

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use chrono::{DateTime, TimeZone, Utc};
    use otherworlds_core::clock::Clock;
    use otherworlds_core::error::DomainError;
    use otherworlds_core::repository::{EventRepository, StoredEvent};
    use otherworlds_core::rng::DeterministicRng;
    use otherworlds_inventory::domain::events::{
        ITEM_ADDED_EVENT_TYPE, InventoryEventKind, ItemAdded,
    };
    use serde_json::Value;
    use sqlx::PgPool;
    use std::sync::{Arc, Mutex};
    use tower::ServiceExt;

    /// Well-known item ID used by `MockEventRepository` so that success tests
    /// for remove-item and equip-item can reference an item that actually
    /// exists in the reconstituted inventory.
    const KNOWN_ITEM_ID: Uuid = Uuid::from_u128(0xAAAA_BBBB_CCCC_DDDD_EEEE_FFFF_0000_1111);

    #[derive(Debug)]
    struct FixedClock(DateTime<Utc>);

    impl Clock for FixedClock {
        fn now(&self) -> DateTime<Utc> {
            self.0
        }
    }

    #[derive(Debug)]
    struct MockRng;

    impl DeterministicRng for MockRng {
        fn next_u32_range(&mut self, min: u32, _max: u32) -> u32 {
            min
        }

        fn next_f64(&mut self) -> f64 {
            0.0
        }
    }

    /// Mock repository that returns a single `ItemAdded` event with a valid
    /// serialized payload. Uses the caller-supplied `aggregate_id` and the
    /// well-known `KNOWN_ITEM_ID` so that the reconstituted inventory contains
    /// that item.
    #[derive(Debug)]
    struct MockEventRepository;

    #[async_trait::async_trait]
    impl EventRepository for MockEventRepository {
        async fn load_events(&self, aggregate_id: Uuid) -> Result<Vec<StoredEvent>, DomainError> {
            let fixed_now = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
            Ok(vec![StoredEvent {
                event_id: Uuid::new_v4(),
                aggregate_id,
                event_type: ITEM_ADDED_EVENT_TYPE.to_owned(),
                payload: serde_json::to_value(InventoryEventKind::ItemAdded(ItemAdded {
                    inventory_id: aggregate_id,
                    item_id: KNOWN_ITEM_ID,
                }))
                .expect("ItemAdded serialization is infallible"),
                sequence_number: 1,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            }])
        }

        async fn append_events(
            &self,
            _aggregate_id: Uuid,
            _expected_version: i64,
            _events: &[StoredEvent],
        ) -> Result<(), DomainError> {
            Ok(())
        }
    }

    #[derive(Debug)]
    struct EmptyEventRepository;

    #[async_trait::async_trait]
    impl EventRepository for EmptyEventRepository {
        async fn load_events(&self, _aggregate_id: Uuid) -> Result<Vec<StoredEvent>, DomainError> {
            Ok(vec![])
        }

        async fn append_events(
            &self,
            _aggregate_id: Uuid,
            _expected_version: i64,
            _events: &[StoredEvent],
        ) -> Result<(), DomainError> {
            Ok(())
        }
    }

    #[derive(Debug)]
    struct FailingEventRepository;

    #[async_trait::async_trait]
    impl EventRepository for FailingEventRepository {
        async fn load_events(&self, _aggregate_id: Uuid) -> Result<Vec<StoredEvent>, DomainError> {
            Err(DomainError::Infrastructure("connection refused".into()))
        }

        async fn append_events(
            &self,
            _aggregate_id: Uuid,
            _expected_version: i64,
            _events: &[StoredEvent],
        ) -> Result<(), DomainError> {
            Err(DomainError::Infrastructure("connection refused".into()))
        }
    }

    fn app_state_with(event_repository: Arc<dyn EventRepository>) -> AppState {
        let pool = PgPool::connect_lazy("postgres://localhost/test").unwrap();
        let clock: Arc<dyn Clock + Send + Sync> = Arc::new(FixedClock(
            Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
        ));
        let rng: Arc<Mutex<dyn DeterministicRng + Send>> = Arc::new(Mutex::new(MockRng));
        AppState::new(pool, clock, rng, event_repository)
    }

    fn test_app_state() -> AppState {
        app_state_with(Arc::new(MockEventRepository))
    }

    fn empty_app_state() -> AppState {
        app_state_with(Arc::new(EmptyEventRepository))
    }

    fn failing_app_state() -> AppState {
        app_state_with(Arc::new(FailingEventRepository))
    }

    #[tokio::test]
    async fn test_add_item_returns_200_with_event_ids() {
        // Arrange
        let app = router().with_state(test_app_state());
        let inventory_id = Uuid::new_v4();
        let item_id = Uuid::new_v4();
        let body = serde_json::json!({
            "inventory_id": inventory_id,
            "item_id": item_id
        });

        let request = Request::builder()
            .method("POST")
            .uri("/add-item")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body_bytes).unwrap();

        let returned_id = Uuid::parse_str(json["aggregate_id"].as_str().unwrap()).unwrap();
        assert_eq!(returned_id, inventory_id);

        let event_ids = json["event_ids"].as_array().unwrap();
        assert_eq!(event_ids.len(), 1);
        for id in event_ids {
            Uuid::parse_str(id.as_str().unwrap()).unwrap();
        }
    }

    #[tokio::test]
    async fn test_remove_item_returns_200_with_event_ids() {
        // Arrange — use KNOWN_ITEM_ID so the item exists in the reconstituted
        // inventory after the mock's ItemAdded event is applied.
        let app = router().with_state(test_app_state());
        let inventory_id = Uuid::new_v4();
        let body = serde_json::json!({
            "inventory_id": inventory_id,
            "item_id": KNOWN_ITEM_ID
        });

        let request = Request::builder()
            .method("POST")
            .uri("/remove-item")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body_bytes).unwrap();

        let returned_id = Uuid::parse_str(json["aggregate_id"].as_str().unwrap()).unwrap();
        assert_eq!(returned_id, inventory_id);

        let event_ids = json["event_ids"].as_array().unwrap();
        assert_eq!(event_ids.len(), 1);
        for id in event_ids {
            Uuid::parse_str(id.as_str().unwrap()).unwrap();
        }
    }

    #[tokio::test]
    async fn test_equip_item_returns_200_with_event_ids() {
        // Arrange — use KNOWN_ITEM_ID so the item exists in the reconstituted
        // inventory after the mock's ItemAdded event is applied.
        let app = router().with_state(test_app_state());
        let inventory_id = Uuid::new_v4();
        let body = serde_json::json!({
            "inventory_id": inventory_id,
            "item_id": KNOWN_ITEM_ID
        });

        let request = Request::builder()
            .method("POST")
            .uri("/equip-item")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body_bytes).unwrap();

        let returned_id = Uuid::parse_str(json["aggregate_id"].as_str().unwrap()).unwrap();
        assert_eq!(returned_id, inventory_id);

        let event_ids = json["event_ids"].as_array().unwrap();
        assert_eq!(event_ids.len(), 1);
        for id in event_ids {
            Uuid::parse_str(id.as_str().unwrap()).unwrap();
        }
    }

    #[tokio::test]
    async fn test_add_item_returns_422_for_missing_body() {
        // Arrange
        let app = router().with_state(test_app_state());

        let request = Request::builder()
            .method("POST")
            .uri("/add-item")
            .header("content-type", "application/json")
            .body(Body::from("{}"))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert — Axum returns 422 for deserialization failures.
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_add_item_returns_404_when_inventory_not_found() {
        // Arrange
        let app = router().with_state(empty_app_state());
        let inventory_id = Uuid::new_v4();
        let item_id = Uuid::new_v4();
        let body = serde_json::json!({
            "inventory_id": inventory_id,
            "item_id": item_id
        });

        let request = Request::builder()
            .method("POST")
            .uri("/add-item")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(json["error"], "aggregate_not_found");
    }

    #[tokio::test]
    async fn test_remove_item_returns_404_when_inventory_not_found() {
        // Arrange
        let app = router().with_state(empty_app_state());
        let inventory_id = Uuid::new_v4();
        let item_id = Uuid::new_v4();
        let body = serde_json::json!({
            "inventory_id": inventory_id,
            "item_id": item_id
        });

        let request = Request::builder()
            .method("POST")
            .uri("/remove-item")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(json["error"], "aggregate_not_found");
    }

    #[tokio::test]
    async fn test_add_item_returns_500_when_repository_fails() {
        // Arrange
        let app = router().with_state(failing_app_state());
        let inventory_id = Uuid::new_v4();
        let item_id = Uuid::new_v4();
        let body = serde_json::json!({
            "inventory_id": inventory_id,
            "item_id": item_id
        });

        let request = Request::builder()
            .method("POST")
            .uri("/add-item")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(json["error"], "infrastructure_error");
    }

    #[tokio::test]
    async fn test_equip_item_returns_404_when_inventory_not_found() {
        // Arrange
        let app = router().with_state(empty_app_state());
        let inventory_id = Uuid::new_v4();
        let item_id = Uuid::new_v4();
        let body = serde_json::json!({
            "inventory_id": inventory_id,
            "item_id": item_id
        });

        let request = Request::builder()
            .method("POST")
            .uri("/equip-item")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(json["error"], "aggregate_not_found");
    }

    #[tokio::test]
    async fn test_remove_item_returns_400_when_item_not_in_inventory() {
        // Arrange — inventory exists (mock returns an ItemAdded for KNOWN_ITEM_ID)
        // but we request removal of a different item.
        let app = router().with_state(test_app_state());
        let inventory_id = Uuid::new_v4();
        let unknown_item_id = Uuid::new_v4();
        let body = serde_json::json!({
            "inventory_id": inventory_id,
            "item_id": unknown_item_id
        });

        let request = Request::builder()
            .method("POST")
            .uri("/remove-item")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(json["error"], "validation_error");
    }
}
