//! Command handlers for the Content Authoring context.
//!
//! This module contains application-level command handler functions that
//! orchestrate domain logic: load aggregate, execute command, persist events.

use otherworlds_core::aggregate::AggregateRoot;
use otherworlds_core::clock::Clock;
use otherworlds_core::error::DomainError;
use otherworlds_core::event::DomainEvent;
use otherworlds_core::repository::{EventRepository, StoredEvent};
use uuid::Uuid;

use crate::domain::aggregates::Campaign;
use crate::domain::commands::{CompileCampaign, IngestCampaign, ValidateCampaign};
use crate::domain::events::{ContentEvent, ContentEventKind};

/// Result of a successfully handled command.
#[derive(Debug)]
pub struct ContentCommandResult {
    /// The aggregate ID affected by the command.
    pub aggregate_id: Uuid,
    /// The stored events produced and persisted.
    pub stored_events: Vec<StoredEvent>,
}

fn to_stored_event(event: &ContentEvent) -> StoredEvent {
    let meta = event.metadata();
    StoredEvent {
        event_id: meta.event_id,
        aggregate_id: meta.aggregate_id,
        event_type: event.event_type().to_owned(),
        payload: event.to_payload(),
        sequence_number: meta.sequence_number,
        correlation_id: meta.correlation_id,
        causation_id: meta.causation_id,
        occurred_at: meta.occurred_at,
    }
}

/// Reconstitutes a `Campaign` from stored events.
///
/// # Errors
///
/// Returns `DomainError::Infrastructure` if event deserialization fails.
pub(crate) fn reconstitute(
    campaign_id: Uuid,
    existing_events: &[StoredEvent],
) -> Result<Campaign, DomainError> {
    let mut campaign = Campaign::new(campaign_id);
    for stored in existing_events {
        let kind: ContentEventKind =
            serde_json::from_value(stored.payload.clone()).map_err(|e| {
                DomainError::Infrastructure(format!("event deserialization failed: {e}"))
            })?;
        let event = ContentEvent {
            metadata: otherworlds_core::event::EventMetadata {
                event_id: stored.event_id,
                event_type: stored.event_type.clone(),
                aggregate_id: stored.aggregate_id,
                sequence_number: stored.sequence_number,
                correlation_id: stored.correlation_id,
                causation_id: stored.causation_id,
                occurred_at: stored.occurred_at,
            },
            kind,
        };
        campaign.apply(&event);
    }
    Ok(campaign)
}

/// Handles the `IngestCampaign` command: creates a new aggregate, ingests the
/// campaign, and persists the resulting events.
///
/// This is a CREATION command — the handler generates the `campaign_id`.
///
/// # Errors
///
/// Returns `DomainError` if event appending fails.
pub async fn handle_ingest_campaign(
    command: &IngestCampaign,
    clock: &dyn Clock,
    repo: &dyn EventRepository,
) -> Result<ContentCommandResult, DomainError> {
    let campaign_id = Uuid::new_v4();
    let mut campaign = Campaign::new(campaign_id);

    campaign.ingest_campaign(&command.source, command.correlation_id, clock)?;

    let stored_events: Vec<StoredEvent> = campaign
        .uncommitted_events()
        .iter()
        .map(to_stored_event)
        .collect();

    repo.append_events(campaign_id, campaign.version(), &stored_events)
        .await?;

    Ok(ContentCommandResult {
        aggregate_id: campaign_id,
        stored_events,
    })
}

/// Handles the `ValidateCampaign` command: loads the aggregate, validates it,
/// and persists the resulting events.
///
/// # Errors
///
/// Returns `DomainError` if event loading or appending fails.
pub async fn handle_validate_campaign(
    command: &ValidateCampaign,
    clock: &dyn Clock,
    repo: &dyn EventRepository,
) -> Result<ContentCommandResult, DomainError> {
    let existing_events = repo.load_events(command.campaign_id).await?;
    if existing_events.is_empty() {
        return Err(DomainError::AggregateNotFound(command.campaign_id));
    }
    let mut campaign = reconstitute(command.campaign_id, &existing_events)?;

    campaign.validate_campaign(command.correlation_id, clock)?;

    let stored_events: Vec<StoredEvent> = campaign
        .uncommitted_events()
        .iter()
        .map(to_stored_event)
        .collect();

    repo.append_events(command.campaign_id, campaign.version(), &stored_events)
        .await?;

    Ok(ContentCommandResult {
        aggregate_id: command.campaign_id,
        stored_events,
    })
}

/// Handles the `CompileCampaign` command: loads the aggregate, compiles it,
/// and persists the resulting events.
///
/// # Errors
///
/// Returns `DomainError` if event loading or appending fails.
pub async fn handle_compile_campaign(
    command: &CompileCampaign,
    clock: &dyn Clock,
    repo: &dyn EventRepository,
) -> Result<ContentCommandResult, DomainError> {
    let existing_events = repo.load_events(command.campaign_id).await?;
    if existing_events.is_empty() {
        return Err(DomainError::AggregateNotFound(command.campaign_id));
    }
    let mut campaign = reconstitute(command.campaign_id, &existing_events)?;

    campaign.compile_campaign(command.correlation_id, clock)?;

    let stored_events: Vec<StoredEvent> = campaign
        .uncommitted_events()
        .iter()
        .map(to_stored_event)
        .collect();

    repo.append_events(command.campaign_id, campaign.version(), &stored_events)
        .await?;

    Ok(ContentCommandResult {
        aggregate_id: command.campaign_id,
        stored_events,
    })
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, TimeZone, Utc};
    use otherworlds_core::error::DomainError;
    use otherworlds_core::repository::StoredEvent;
    use uuid::Uuid;

    use crate::application::command_handlers::{
        handle_compile_campaign, handle_ingest_campaign, handle_validate_campaign,
    };
    use crate::domain::commands::{CompileCampaign, IngestCampaign, ValidateCampaign};
    use crate::domain::events::{
        CAMPAIGN_COMPILED_EVENT_TYPE, CAMPAIGN_INGESTED_EVENT_TYPE, CAMPAIGN_VALIDATED_EVENT_TYPE,
        CampaignIngested, CampaignValidated, ContentEventKind,
    };
    use otherworlds_test_support::{FixedClock, RecordingEventRepository};

    fn dummy_ingested_event(aggregate_id: Uuid, fixed_now: DateTime<Utc>) -> StoredEvent {
        StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            event_type: CAMPAIGN_INGESTED_EVENT_TYPE.to_owned(),
            payload: serde_json::to_value(ContentEventKind::CampaignIngested(CampaignIngested {
                campaign_id: aggregate_id,
                version_hash: "abc123".to_owned(),
            }))
            .unwrap(),
            sequence_number: 1,
            correlation_id: Uuid::new_v4(),
            causation_id: Uuid::new_v4(),
            occurred_at: fixed_now,
        }
    }

    fn dummy_validated_event(aggregate_id: Uuid, fixed_now: DateTime<Utc>) -> StoredEvent {
        StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            event_type: CAMPAIGN_VALIDATED_EVENT_TYPE.to_owned(),
            payload: serde_json::to_value(ContentEventKind::CampaignValidated(CampaignValidated {
                campaign_id: aggregate_id,
            }))
            .unwrap(),
            sequence_number: 2,
            correlation_id: Uuid::new_v4(),
            causation_id: Uuid::new_v4(),
            occurred_at: fixed_now,
        }
    }

    #[tokio::test]
    async fn test_handle_ingest_campaign_creates_new_aggregate() {
        // Arrange
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let repo = RecordingEventRepository::new(Ok(Vec::new()));

        let command = IngestCampaign {
            correlation_id,
            source: "# My Campaign".to_owned(),
        };

        // Act
        let result = handle_ingest_campaign(&command, &clock, &repo).await;

        // Assert
        let cmd_result = result.unwrap();
        assert_eq!(cmd_result.stored_events.len(), 1);

        let appended = repo.appended_events();
        assert_eq!(appended.len(), 1);

        let (agg_id, expected_version, events) = &appended[0];
        assert_eq!(*agg_id, cmd_result.aggregate_id);
        assert_eq!(*expected_version, 0);
        assert_eq!(events.len(), 1);

        let stored = &events[0];
        assert_eq!(stored.event_type, CAMPAIGN_INGESTED_EVENT_TYPE);
        assert_eq!(stored.aggregate_id, cmd_result.aggregate_id);
        assert_eq!(stored.sequence_number, 1);
        assert_eq!(stored.correlation_id, correlation_id);
        assert_eq!(stored.causation_id, correlation_id);
        assert_eq!(stored.occurred_at, fixed_now);
    }

    #[tokio::test]
    async fn test_handle_validate_campaign_persists_validated_event() {
        // Arrange
        let campaign_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let existing_event = dummy_ingested_event(campaign_id, fixed_now);
        let repo = RecordingEventRepository::new(Ok(vec![existing_event]));

        let command = ValidateCampaign {
            correlation_id,
            campaign_id,
        };

        // Act
        let result = handle_validate_campaign(&command, &clock, &repo).await;

        // Assert
        let cmd_result = result.unwrap();
        assert_eq!(cmd_result.aggregate_id, campaign_id);
        assert_eq!(cmd_result.stored_events.len(), 1);

        let appended = repo.appended_events();
        assert_eq!(appended.len(), 1);

        let (agg_id, expected_version, events) = &appended[0];
        assert_eq!(*agg_id, campaign_id);
        assert_eq!(*expected_version, 1);
        assert_eq!(events.len(), 1);

        let stored = &events[0];
        assert_eq!(stored.event_type, CAMPAIGN_VALIDATED_EVENT_TYPE);
        assert_eq!(stored.aggregate_id, campaign_id);
        assert_eq!(stored.sequence_number, 2);
        assert_eq!(stored.correlation_id, correlation_id);
        assert_eq!(stored.causation_id, correlation_id);
        assert_eq!(stored.occurred_at, fixed_now);
    }

    #[tokio::test]
    async fn test_handle_compile_campaign_persists_compiled_event() {
        // Arrange
        let campaign_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let ingested = dummy_ingested_event(campaign_id, fixed_now);
        let validated = dummy_validated_event(campaign_id, fixed_now);
        let repo = RecordingEventRepository::new(Ok(vec![ingested, validated]));

        let command = CompileCampaign {
            correlation_id,
            campaign_id,
        };

        // Act
        let result = handle_compile_campaign(&command, &clock, &repo).await;

        // Assert
        let cmd_result = result.unwrap();
        assert_eq!(cmd_result.aggregate_id, campaign_id);
        assert_eq!(cmd_result.stored_events.len(), 1);

        let appended = repo.appended_events();
        assert_eq!(appended.len(), 1);

        let (agg_id, expected_version, events) = &appended[0];
        assert_eq!(*agg_id, campaign_id);
        assert_eq!(*expected_version, 2);
        assert_eq!(events.len(), 1);

        let stored = &events[0];
        assert_eq!(stored.event_type, CAMPAIGN_COMPILED_EVENT_TYPE);
        assert_eq!(stored.aggregate_id, campaign_id);
        assert_eq!(stored.sequence_number, 3);
        assert_eq!(stored.correlation_id, correlation_id);
        assert_eq!(stored.causation_id, correlation_id);
        assert_eq!(stored.occurred_at, fixed_now);
    }

    #[tokio::test]
    async fn test_handle_validate_campaign_returns_error_when_not_found() {
        // Arrange
        let campaign_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let repo = RecordingEventRepository::new(Ok(Vec::new()));

        let command = ValidateCampaign {
            correlation_id,
            campaign_id,
        };

        // Act
        let result = handle_validate_campaign(&command, &clock, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::AggregateNotFound(id) => assert_eq!(id, campaign_id),
            other => panic!("expected AggregateNotFound, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_compile_campaign_returns_error_when_not_found() {
        // Arrange
        let campaign_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let repo = RecordingEventRepository::new(Ok(Vec::new()));

        let command = CompileCampaign {
            correlation_id,
            campaign_id,
        };

        // Act
        let result = handle_compile_campaign(&command, &clock, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::AggregateNotFound(id) => assert_eq!(id, campaign_id),
            other => panic!("expected AggregateNotFound, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_handle_compile_campaign_returns_error_when_not_validated() {
        // Arrange — campaign is ingested but not validated.
        let campaign_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let ingested = dummy_ingested_event(campaign_id, fixed_now);
        let repo = RecordingEventRepository::new(Ok(vec![ingested]));

        let command = CompileCampaign {
            correlation_id,
            campaign_id,
        };

        // Act
        let result = handle_compile_campaign(&command, &clock, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert!(msg.contains(&campaign_id.to_string()));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }
}
