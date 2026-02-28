//! Integration tests for `PgEventRepository`.

use chrono::Utc;
use otherworlds_core::error::DomainError;
use otherworlds_core::repository::{EventRepository, StoredEvent};
use otherworlds_event_store::pg_event_repository::PgEventRepository;
use sqlx::PgPool;
use uuid::Uuid;

/// Helper to build a `StoredEvent` with sensible defaults.
fn make_stored_event(aggregate_id: Uuid, sequence_number: i64) -> StoredEvent {
    StoredEvent {
        event_id: Uuid::new_v4(),
        aggregate_id,
        event_type: "TestEvent".to_string(),
        payload: serde_json::json!({"key": "value"}),
        sequence_number,
        correlation_id: Uuid::new_v4(),
        causation_id: Uuid::new_v4(),
        occurred_at: Utc::now(),
    }
}

// --- load_events ---

#[sqlx::test(migrations = "../../migrations")]
async fn test_load_events_returns_empty_vec_for_nonexistent_aggregate(pool: PgPool) {
    let repo = PgEventRepository::new(pool);
    let aggregate_id = Uuid::new_v4();

    let events = repo.load_events(aggregate_id).await.unwrap();

    assert!(events.is_empty());
}

// --- append_events + load_events round-trip ---

#[sqlx::test(migrations = "../../migrations")]
async fn test_append_and_load_single_event(pool: PgPool) {
    let repo = PgEventRepository::new(pool);
    let aggregate_id = Uuid::new_v4();
    let event = make_stored_event(aggregate_id, 1);
    let expected_event_id = event.event_id;
    let expected_event_type = event.event_type.clone();
    let expected_payload = event.payload.clone();
    let expected_correlation_id = event.correlation_id;
    let expected_causation_id = event.causation_id;
    let expected_occurred_at = event.occurred_at;

    repo.append_events(aggregate_id, 0, &[event]).await.unwrap();

    let loaded = repo.load_events(aggregate_id).await.unwrap();
    assert_eq!(loaded.len(), 1);

    let e = &loaded[0];
    assert_eq!(e.event_id, expected_event_id);
    assert_eq!(e.aggregate_id, aggregate_id);
    assert_eq!(e.event_type, expected_event_type);
    assert_eq!(e.payload, expected_payload);
    assert_eq!(e.sequence_number, 1);
    assert_eq!(e.correlation_id, expected_correlation_id);
    assert_eq!(e.causation_id, expected_causation_id);
    assert_eq!(e.occurred_at, expected_occurred_at);
}

// --- ordering ---

#[sqlx::test(migrations = "../../migrations")]
async fn test_append_multiple_events_preserves_sequence_order(pool: PgPool) {
    let repo = PgEventRepository::new(pool);
    let aggregate_id = Uuid::new_v4();
    let events = vec![
        make_stored_event(aggregate_id, 1),
        make_stored_event(aggregate_id, 2),
        make_stored_event(aggregate_id, 3),
    ];

    repo.append_events(aggregate_id, 0, &events).await.unwrap();

    let loaded = repo.load_events(aggregate_id).await.unwrap();
    assert_eq!(loaded.len(), 3);
    assert_eq!(loaded[0].sequence_number, 1);
    assert_eq!(loaded[1].sequence_number, 2);
    assert_eq!(loaded[2].sequence_number, 3);
}

// --- aggregate isolation ---

#[sqlx::test(migrations = "../../migrations")]
async fn test_aggregate_isolation(pool: PgPool) {
    let repo = PgEventRepository::new(pool);
    let agg_a = Uuid::new_v4();
    let agg_b = Uuid::new_v4();

    repo.append_events(agg_a, 0, &[make_stored_event(agg_a, 1)])
        .await
        .unwrap();
    repo.append_events(agg_b, 0, &[make_stored_event(agg_b, 1)])
        .await
        .unwrap();

    let loaded_a = repo.load_events(agg_a).await.unwrap();
    let loaded_b = repo.load_events(agg_b).await.unwrap();

    assert_eq!(loaded_a.len(), 1);
    assert_eq!(loaded_b.len(), 1);
    assert_eq!(loaded_a[0].aggregate_id, agg_a);
    assert_eq!(loaded_b[0].aggregate_id, agg_b);
}

// --- concurrency ---

#[sqlx::test(migrations = "../../migrations")]
async fn test_concurrency_conflict_on_duplicate_sequence_number(pool: PgPool) {
    let repo = PgEventRepository::new(pool);
    let aggregate_id = Uuid::new_v4();

    // First append succeeds.
    repo.append_events(aggregate_id, 0, &[make_stored_event(aggregate_id, 1)])
        .await
        .unwrap();

    // Second append with same sequence_number should fail.
    let result = repo
        .append_events(aggregate_id, 0, &[make_stored_event(aggregate_id, 1)])
        .await;

    match result {
        Err(DomainError::ConcurrencyConflict {
            aggregate_id: conflict_agg_id,
            expected,
            actual,
        }) => {
            assert_eq!(conflict_agg_id, aggregate_id);
            assert_eq!(expected, 0);
            assert_eq!(actual, 1);
        }
        other => panic!("expected ConcurrencyConflict, got {other:?}"),
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_stale_expected_version_with_non_overlapping_sequences(pool: PgPool) {
    let repo = PgEventRepository::new(pool);
    let aggregate_id = Uuid::new_v4();

    // Append events 1-2 with expected_version 0.
    repo.append_events(
        aggregate_id,
        0,
        &[
            make_stored_event(aggregate_id, 1),
            make_stored_event(aggregate_id, 2),
        ],
    )
    .await
    .unwrap();

    // Attempt to append events 3-4 with stale expected_version 0 (actual is 2).
    // Sequence numbers don't collide, but the version check must still reject.
    let result = repo
        .append_events(
            aggregate_id,
            0,
            &[
                make_stored_event(aggregate_id, 3),
                make_stored_event(aggregate_id, 4),
            ],
        )
        .await;

    match result {
        Err(DomainError::ConcurrencyConflict {
            aggregate_id: conflict_agg_id,
            expected,
            actual,
        }) => {
            assert_eq!(conflict_agg_id, aggregate_id);
            assert_eq!(expected, 0);
            assert_eq!(actual, 2);
        }
        other => panic!("expected ConcurrencyConflict, got {other:?}"),
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_sequential_appends_with_correct_expected_version(pool: PgPool) {
    let repo = PgEventRepository::new(pool);
    let aggregate_id = Uuid::new_v4();

    // First batch: events 1-2, expected version 0.
    repo.append_events(
        aggregate_id,
        0,
        &[
            make_stored_event(aggregate_id, 1),
            make_stored_event(aggregate_id, 2),
        ],
    )
    .await
    .unwrap();

    // Second batch: events 3-4, expected version 2.
    repo.append_events(
        aggregate_id,
        2,
        &[
            make_stored_event(aggregate_id, 3),
            make_stored_event(aggregate_id, 4),
        ],
    )
    .await
    .unwrap();

    let loaded = repo.load_events(aggregate_id).await.unwrap();
    assert_eq!(loaded.len(), 4);
    for (i, event) in loaded.iter().enumerate() {
        assert_eq!(event.sequence_number, i64::try_from(i + 1).unwrap());
    }
}

// --- edge cases ---

#[sqlx::test(migrations = "../../migrations")]
async fn test_append_empty_events_is_noop(pool: PgPool) {
    let repo = PgEventRepository::new(pool);
    let aggregate_id = Uuid::new_v4();

    repo.append_events(aggregate_id, 0, &[]).await.unwrap();

    let loaded = repo.load_events(aggregate_id).await.unwrap();
    assert!(loaded.is_empty());
}

// --- payload serialization ---

#[sqlx::test(migrations = "../../migrations")]
async fn test_complex_json_payload_round_trip(pool: PgPool) {
    let repo = PgEventRepository::new(pool);
    let aggregate_id = Uuid::new_v4();
    let complex_payload = serde_json::json!({
        "nested": {"key": "value", "number": 42},
        "array": [1, "two", null, true, false],
        "null_field": null,
        "boolean": true,
        "empty_object": {},
        "empty_array": []
    });

    let mut event = make_stored_event(aggregate_id, 1);
    event.payload = complex_payload.clone();

    repo.append_events(aggregate_id, 0, &[event]).await.unwrap();

    let loaded = repo.load_events(aggregate_id).await.unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].payload, complex_payload);
}

// --- timestamp precision ---

#[sqlx::test(migrations = "../../migrations")]
async fn test_timestamp_precision(pool: PgPool) {
    let repo = PgEventRepository::new(pool);
    let aggregate_id = Uuid::new_v4();
    let event = make_stored_event(aggregate_id, 1);
    let original_timestamp = event.occurred_at;

    repo.append_events(aggregate_id, 0, &[event]).await.unwrap();

    let loaded = repo.load_events(aggregate_id).await.unwrap();
    assert_eq!(loaded.len(), 1);

    // PostgreSQL TIMESTAMPTZ has microsecond precision.
    let original_micros = original_timestamp.timestamp_micros();
    let loaded_micros = loaded[0].occurred_at.timestamp_micros();
    assert_eq!(original_micros, loaded_micros);
}
