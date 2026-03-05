//! Branching — context-agnostic event cloning for timeline forks.

use serde_json::Value;
use uuid::Uuid;

use crate::clock::Clock;
use crate::repository::StoredEvent;
use crate::rng::DeterministicRng;

/// Clones a sequence of `StoredEvent`s for a branch, rewriting IDs so the
/// cloned events belong to a new aggregate. Replaces occurrences of
/// `source_aggregate_id` in JSON payloads with `new_aggregate_id`.
///
/// Returns new `StoredEvent`s with fresh event IDs, sequential numbering
/// starting at `start_sequence`, and the provided `correlation_id`.
pub fn clone_events_for_branch(
    source_events: &[StoredEvent],
    source_aggregate_id: Uuid,
    new_aggregate_id: Uuid,
    correlation_id: Uuid,
    start_sequence: i64,
    clock: &dyn Clock,
    rng: &mut dyn DeterministicRng,
) -> Vec<StoredEvent> {
    source_events
        .iter()
        .enumerate()
        .map(|(i, event)| {
            #[allow(clippy::cast_possible_wrap)]
            let sequence_number = start_sequence + i as i64;
            let payload =
                rewrite_uuid_in_value(&event.payload, source_aggregate_id, new_aggregate_id);
            StoredEvent {
                event_id: rng.next_uuid(),
                aggregate_id: new_aggregate_id,
                event_type: event.event_type.clone(),
                payload,
                sequence_number,
                correlation_id,
                causation_id: correlation_id,
                occurred_at: clock.now(),
            }
        })
        .collect()
}

/// Recursively walks a `serde_json::Value` and replaces any string that
/// matches `old_id` with `new_id`.
fn rewrite_uuid_in_value(value: &Value, old_id: Uuid, new_id: Uuid) -> Value {
    let old_str = old_id.to_string();
    let new_str = new_id.to_string();
    match value {
        Value::String(s) if *s == old_str => Value::String(new_str),
        Value::Object(map) => {
            let new_map: serde_json::Map<String, Value> = map
                .iter()
                .map(|(k, v)| (k.clone(), rewrite_uuid_in_value(v, old_id, new_id)))
                .collect();
            Value::Object(new_map)
        }
        Value::Array(arr) => {
            Value::Array(arr.iter().map(|v| rewrite_uuid_in_value(v, old_id, new_id)).collect())
        }
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use serde_json::json;
    use uuid::Uuid;

    use super::*;

    #[derive(Debug)]
    struct FixedClock(chrono::DateTime<chrono::Utc>);
    impl Clock for FixedClock {
        fn now(&self) -> chrono::DateTime<chrono::Utc> {
            self.0
        }
    }

    #[derive(Debug)]
    struct SeqRng(u32);
    impl DeterministicRng for SeqRng {
        fn next_u32_range(&mut self, _min: u32, _max: u32) -> u32 {
            self.0 += 1;
            self.0
        }
        fn next_f64(&mut self) -> f64 {
            self.0 += 1;
            f64::from(self.0) / 1000.0
        }
    }

    fn make_stored_event(aggregate_id: Uuid, seq: i64, payload: Value) -> StoredEvent {
        StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            event_type: "test.event".to_owned(),
            payload,
            sequence_number: seq,
            correlation_id: Uuid::new_v4(),
            causation_id: Uuid::new_v4(),
            occurred_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
        }
    }

    #[test]
    fn test_clone_events_rewrites_aggregate_id_in_metadata() {
        // Arrange
        let source_id = Uuid::new_v4();
        let new_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 3, 1, 12, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut rng = SeqRng(0);
        let source_events = vec![
            make_stored_event(source_id, 1, json!({"session_id": source_id.to_string()})),
        ];

        // Act
        let cloned = clone_events_for_branch(
            &source_events,
            source_id,
            new_id,
            correlation_id,
            1,
            &clock,
            &mut rng,
        );

        // Assert
        assert_eq!(cloned.len(), 1);
        assert_eq!(cloned[0].aggregate_id, new_id);
        assert_eq!(cloned[0].correlation_id, correlation_id);
        assert_eq!(cloned[0].causation_id, correlation_id);
        assert_eq!(cloned[0].occurred_at, fixed_now);
        assert_eq!(cloned[0].sequence_number, 1);
        assert_ne!(cloned[0].event_id, source_events[0].event_id);
    }

    #[test]
    fn test_clone_events_rewrites_uuid_in_payload() {
        // Arrange
        let source_id = Uuid::new_v4();
        let new_id = Uuid::new_v4();
        let other_uuid = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 3, 1, 12, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut rng = SeqRng(0);
        let payload = json!({
            "character_id": source_id.to_string(),
            "name": "Gandalf",
            "other_id": other_uuid.to_string(),
        });
        let source_events = vec![make_stored_event(source_id, 1, payload)];

        // Act
        let cloned = clone_events_for_branch(
            &source_events,
            source_id,
            new_id,
            Uuid::new_v4(),
            1,
            &clock,
            &mut rng,
        );

        // Assert — source_id replaced, other_uuid untouched, name untouched
        let p = &cloned[0].payload;
        assert_eq!(p["character_id"], new_id.to_string());
        assert_eq!(p["name"], "Gandalf");
        assert_eq!(p["other_id"], other_uuid.to_string());
    }

    #[test]
    fn test_clone_events_sequences_from_start() {
        // Arrange
        let source_id = Uuid::new_v4();
        let new_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 3, 1, 12, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut rng = SeqRng(0);
        let source_events = vec![
            make_stored_event(source_id, 1, json!({})),
            make_stored_event(source_id, 2, json!({})),
            make_stored_event(source_id, 3, json!({})),
        ];

        // Act
        let cloned = clone_events_for_branch(
            &source_events,
            source_id,
            new_id,
            Uuid::new_v4(),
            5,
            &clock,
            &mut rng,
        );

        // Assert
        assert_eq!(cloned.len(), 3);
        assert_eq!(cloned[0].sequence_number, 5);
        assert_eq!(cloned[1].sequence_number, 6);
        assert_eq!(cloned[2].sequence_number, 7);
    }

    #[test]
    fn test_clone_events_handles_nested_payload() {
        // Arrange
        let source_id = Uuid::new_v4();
        let new_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 3, 1, 12, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut rng = SeqRng(0);
        let payload = json!({
            "outer": {
                "inner_id": source_id.to_string(),
                "items": [source_id.to_string(), "other"],
            }
        });
        let source_events = vec![make_stored_event(source_id, 1, payload)];

        // Act
        let cloned = clone_events_for_branch(
            &source_events,
            source_id,
            new_id,
            Uuid::new_v4(),
            1,
            &clock,
            &mut rng,
        );

        // Assert
        let p = &cloned[0].payload;
        assert_eq!(p["outer"]["inner_id"], new_id.to_string());
        assert_eq!(p["outer"]["items"][0], new_id.to_string());
        assert_eq!(p["outer"]["items"][1], "other");
    }

    #[test]
    fn test_clone_empty_events_returns_empty() {
        let fixed_now = Utc.with_ymd_and_hms(2026, 3, 1, 12, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut rng = SeqRng(0);

        let cloned = clone_events_for_branch(
            &[],
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            1,
            &clock,
            &mut rng,
        );

        assert!(cloned.is_empty());
    }
}
