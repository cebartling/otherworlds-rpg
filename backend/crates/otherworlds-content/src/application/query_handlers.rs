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
    /// The campaign version hash.
    pub version_hash: Option<String>,
    /// Current version (event count).
    pub version: i64,
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

    use crate::application::query_handlers::get_campaign_by_id;
    use crate::domain::events::{
        CAMPAIGN_INGESTED_EVENT_TYPE, CampaignIngested, ContentEventKind,
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
}
