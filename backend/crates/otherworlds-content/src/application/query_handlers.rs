//! Query handlers for the Content Authoring context.
//!
//! This module contains query handlers that reconstitute aggregates
//! from stored events and return read-only view DTOs.

use otherworlds_core::error::DomainError;
use otherworlds_core::repository::EventRepository;
use serde::Serialize;
use uuid::Uuid;

use crate::application::command_handlers;

/// Read-only view of a campaign aggregate.
#[derive(Debug, Serialize)]
pub struct CampaignView {
    /// The campaign identifier.
    pub campaign_id: Uuid,
    /// Whether the campaign has been ingested.
    pub ingested: bool,
    /// Whether the campaign has been validated.
    pub validated: bool,
    /// Whether the campaign has been compiled into runtime format.
    pub compiled: bool,
    /// The campaign version hash.
    pub version_hash: Option<String>,
    /// Current version (event count).
    pub version: i64,
}

/// Event types used by the Content Authoring context.
const EVENT_TYPES: &[&str] = &[
    "content.campaign_ingested",
    "content.campaign_validated",
    "content.campaign_compiled",
    "content.campaign_archived",
];

/// Summary view for listing campaigns.
#[derive(Debug, Serialize)]
pub struct CampaignSummary {
    /// The campaign identifier.
    pub campaign_id: Uuid,
    /// Whether the campaign has been ingested.
    pub ingested: bool,
    /// Whether the campaign has been validated.
    pub validated: bool,
    /// Whether the campaign has been compiled into runtime format.
    pub compiled: bool,
    /// Current version (event count).
    pub version: i64,
}

/// Lists all campaigns.
///
/// # Errors
///
/// Returns `DomainError::Infrastructure` if querying or deserialization fails.
pub async fn list_campaigns(
    repo: &dyn EventRepository,
) -> Result<Vec<CampaignSummary>, DomainError> {
    let ids = repo.list_aggregate_ids(EVENT_TYPES).await?;
    let mut summaries = Vec::with_capacity(ids.len());
    for id in ids {
        let stored_events = repo.load_events(id).await?;
        if stored_events.is_empty() {
            continue;
        }
        let campaign = command_handlers::reconstitute(id, &stored_events)?;
        if campaign.archived {
            continue;
        }
        summaries.push(CampaignSummary {
            campaign_id: id,
            ingested: campaign.ingested,
            validated: campaign.validated,
            compiled: campaign.compiled,
            version: campaign.version,
        });
    }
    Ok(summaries)
}

/// Retrieves a campaign by its aggregate ID.
///
/// # Errors
///
/// Returns `DomainError::AggregateNotFound` if no events exist for the ID.
/// Returns `DomainError::Infrastructure` if event deserialization fails.
pub async fn get_campaign_by_id(
    campaign_id: Uuid,
    repo: &dyn EventRepository,
) -> Result<CampaignView, DomainError> {
    let stored_events = repo.load_events(campaign_id).await?;
    if stored_events.is_empty() {
        return Err(DomainError::AggregateNotFound(campaign_id));
    }
    let campaign = command_handlers::reconstitute(campaign_id, &stored_events)?;
    Ok(CampaignView {
        campaign_id,
        ingested: campaign.ingested,
        validated: campaign.validated,
        compiled: campaign.compiled,
        version_hash: campaign.version_hash.clone(),
        version: campaign.version,
    })
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use otherworlds_core::error::DomainError;
    use otherworlds_core::repository::StoredEvent;
    use uuid::Uuid;

    use crate::application::query_handlers::{get_campaign_by_id, list_campaigns};
    use crate::domain::events::{
        CAMPAIGN_ARCHIVED_EVENT_TYPE, CAMPAIGN_COMPILED_EVENT_TYPE, CAMPAIGN_INGESTED_EVENT_TYPE,
        CAMPAIGN_VALIDATED_EVENT_TYPE, CampaignArchived, CampaignCompiled, CampaignIngested,
        CampaignValidated, ContentEventKind,
    };
    use otherworlds_test_support::{EmptyEventRepository, RecordingEventRepository};

    #[tokio::test]
    async fn test_get_campaign_by_id_returns_view_with_state() {
        // Arrange
        let campaign_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();

        let events = vec![StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: campaign_id,
            event_type: CAMPAIGN_INGESTED_EVENT_TYPE.to_owned(),
            payload: serde_json::to_value(ContentEventKind::CampaignIngested(CampaignIngested {
                campaign_id,
                version_hash: "abc123".to_owned(),
                source: "# My Campaign".to_owned(),
            }))
            .unwrap(),
            sequence_number: 1,
            correlation_id: Uuid::new_v4(),
            causation_id: Uuid::new_v4(),
            occurred_at: fixed_now,
        }];
        let repo = RecordingEventRepository::new(Ok(events));

        // Act
        let view = get_campaign_by_id(campaign_id, &repo).await.unwrap();

        // Assert
        assert_eq!(view.campaign_id, campaign_id);
        assert!(view.ingested);
        assert!(!view.validated);
        assert!(!view.compiled);
        assert_eq!(view.version_hash, Some("abc123".to_owned()));
        assert_eq!(view.version, 1);
    }

    #[tokio::test]
    async fn test_get_campaign_by_id_returns_not_found_when_no_events() {
        // Arrange
        let campaign_id = Uuid::new_v4();
        let repo = EmptyEventRepository;

        // Act
        let result = get_campaign_by_id(campaign_id, &repo).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::AggregateNotFound(id) => assert_eq!(id, campaign_id),
            other => panic!("expected AggregateNotFound, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_list_campaigns_returns_empty_when_no_aggregates() {
        let repo = EmptyEventRepository;

        let result = list_campaigns(&repo).await.unwrap();

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_list_campaigns_returns_summaries() {
        let campaign_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();

        let events = vec![StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: campaign_id,
            event_type: CAMPAIGN_INGESTED_EVENT_TYPE.to_owned(),
            payload: serde_json::to_value(ContentEventKind::CampaignIngested(CampaignIngested {
                campaign_id,
                version_hash: "abc123".to_owned(),
                source: "# My Campaign".to_owned(),
            }))
            .unwrap(),
            sequence_number: 1,
            correlation_id: Uuid::new_v4(),
            causation_id: Uuid::new_v4(),
            occurred_at: fixed_now,
        }];
        let repo = RecordingEventRepository::with_aggregate_ids(Ok(events), vec![campaign_id]);

        let result = list_campaigns(&repo).await.unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].campaign_id, campaign_id);
        assert!(result[0].ingested);
        assert!(!result[0].validated);
        assert_eq!(result[0].version, 1);
    }

    #[tokio::test]
    async fn test_list_campaigns_excludes_archived() {
        // Arrange
        let campaign_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();

        let events = vec![
            StoredEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: campaign_id,
                event_type: CAMPAIGN_INGESTED_EVENT_TYPE.to_owned(),
                payload: serde_json::to_value(ContentEventKind::CampaignIngested(
                    CampaignIngested {
                        campaign_id,
                        version_hash: "abc123".to_owned(),
                        source: "# My Campaign".to_owned(),
                    },
                ))
                .unwrap(),
                sequence_number: 1,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
            StoredEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: campaign_id,
                event_type: CAMPAIGN_ARCHIVED_EVENT_TYPE.to_owned(),
                payload: serde_json::to_value(ContentEventKind::CampaignArchived(
                    CampaignArchived { campaign_id },
                ))
                .unwrap(),
                sequence_number: 2,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
        ];
        let repo = RecordingEventRepository::with_aggregate_ids(Ok(events), vec![campaign_id]);

        // Act
        let result = list_campaigns(&repo).await.unwrap();

        // Assert
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_get_campaign_by_id_includes_compiled_flag() {
        // Arrange — ingested + validated + compiled
        let campaign_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();

        let events = vec![
            StoredEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: campaign_id,
                event_type: CAMPAIGN_INGESTED_EVENT_TYPE.to_owned(),
                payload: serde_json::to_value(ContentEventKind::CampaignIngested(
                    CampaignIngested {
                        campaign_id,
                        version_hash: "abc123".to_owned(),
                        source: "# My Campaign".to_owned(),
                    },
                ))
                .unwrap(),
                sequence_number: 1,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
            StoredEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: campaign_id,
                event_type: CAMPAIGN_VALIDATED_EVENT_TYPE.to_owned(),
                payload: serde_json::to_value(ContentEventKind::CampaignValidated(
                    CampaignValidated { campaign_id },
                ))
                .unwrap(),
                sequence_number: 2,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
            StoredEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: campaign_id,
                event_type: CAMPAIGN_COMPILED_EVENT_TYPE.to_owned(),
                payload: serde_json::to_value(ContentEventKind::CampaignCompiled(
                    CampaignCompiled {
                        campaign_id,
                        version_hash: "abc123".to_owned(),
                    },
                ))
                .unwrap(),
                sequence_number: 3,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
        ];
        let repo = RecordingEventRepository::new(Ok(events));

        // Act
        let view = get_campaign_by_id(campaign_id, &repo).await.unwrap();

        // Assert
        assert!(view.compiled);
        assert!(view.ingested);
        assert!(view.validated);
    }

    #[tokio::test]
    async fn test_list_campaigns_includes_compiled_flag() {
        // Arrange — ingested-only campaign should have compiled == false.
        let campaign_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();

        let events = vec![StoredEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: campaign_id,
            event_type: CAMPAIGN_INGESTED_EVENT_TYPE.to_owned(),
            payload: serde_json::to_value(ContentEventKind::CampaignIngested(CampaignIngested {
                campaign_id,
                version_hash: "abc123".to_owned(),
                source: "# My Campaign".to_owned(),
            }))
            .unwrap(),
            sequence_number: 1,
            correlation_id: Uuid::new_v4(),
            causation_id: Uuid::new_v4(),
            occurred_at: fixed_now,
        }];
        let repo = RecordingEventRepository::with_aggregate_ids(Ok(events), vec![campaign_id]);

        // Act
        let result = list_campaigns(&repo).await.unwrap();

        // Assert
        assert_eq!(result.len(), 1);
        assert!(!result[0].compiled);
    }
}
