//! Aggregate roots for the Content Authoring context.

use sha2::{Digest, Sha256};

use otherworlds_core::aggregate::AggregateRoot;
use otherworlds_core::clock::Clock;
use otherworlds_core::error::DomainError;
use otherworlds_core::event::EventMetadata;
use uuid::Uuid;

use super::events::{
    CAMPAIGN_COMPILED_EVENT_TYPE, CAMPAIGN_INGESTED_EVENT_TYPE, CAMPAIGN_VALIDATED_EVENT_TYPE,
    CampaignCompiled, CampaignIngested, CampaignValidated, ContentEvent, ContentEventKind,
};

/// The aggregate root for a campaign.
#[derive(Debug)]
pub struct Campaign {
    /// Aggregate identifier.
    pub id: Uuid,
    /// Current version (event count).
    pub(crate) version: i64,
    /// Whether the campaign has been ingested.
    pub(crate) ingested: bool,
    /// Whether the campaign has been validated.
    pub(crate) validated: bool,
    /// The campaign version hash (set on ingestion).
    pub(crate) version_hash: Option<String>,
    /// Uncommitted events pending persistence.
    uncommitted_events: Vec<ContentEvent>,
}

impl Campaign {
    /// Creates a new campaign.
    #[must_use]
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            version: 0,
            ingested: false,
            validated: false,
            version_hash: None,
            uncommitted_events: Vec::new(),
        }
    }

    /// Returns the next sequence number for a new event.
    #[allow(clippy::cast_possible_wrap)]
    fn next_sequence_number(&self) -> i64 {
        self.version + self.uncommitted_events.len() as i64 + 1
    }

    /// Computes a stable version hash from the source content using SHA-256.
    ///
    /// Uses a cryptographic hash to guarantee stability across Rust versions
    /// and platforms, as required by the manifesto's hash-validation constraint.
    fn compute_version_hash(source: &str) -> String {
        let hash = Sha256::digest(source.as_bytes());
        format!("{hash:x}")
    }

    /// Ingests a campaign from source, producing a `CampaignIngested` event.
    ///
    /// # Errors
    ///
    /// Returns `DomainError::Validation` if the campaign has already been ingested
    /// or if the source content is empty.
    pub fn ingest_campaign(
        &mut self,
        source: &str,
        correlation_id: Uuid,
        clock: &dyn Clock,
    ) -> Result<(), DomainError> {
        if source.trim().is_empty() {
            return Err(DomainError::Validation(
                "campaign source content must not be empty".to_owned(),
            ));
        }
        if self.ingested {
            return Err(DomainError::Validation(format!(
                "campaign {} has already been ingested",
                self.id
            )));
        }

        let version_hash = Self::compute_version_hash(source);

        // TODO: event_id uses Uuid::new_v4() which breaks replay determinism.
        // Requires extending DeterministicRng to support UUID generation and
        // threading &mut dyn DeterministicRng through all domain methods.
        let event = ContentEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: CAMPAIGN_INGESTED_EVENT_TYPE.to_owned(),
                aggregate_id: self.id,
                sequence_number: self.next_sequence_number(),
                correlation_id,
                causation_id: correlation_id,
                occurred_at: clock.now(),
            },
            kind: ContentEventKind::CampaignIngested(CampaignIngested {
                campaign_id: self.id,
                version_hash,
            }),
        };

        self.uncommitted_events.push(event);
        Ok(())
    }

    /// Validates a campaign, producing a `CampaignValidated` event.
    ///
    /// TODO: Add actual validation logic — schema checks, reference integrity,
    /// required-field verification, scene graph connectivity, etc.
    ///
    /// # Errors
    ///
    /// Returns `DomainError::Validation` if the campaign has not been ingested.
    pub fn validate_campaign(
        &mut self,
        correlation_id: Uuid,
        clock: &dyn Clock,
    ) -> Result<(), DomainError> {
        if !self.ingested {
            return Err(DomainError::Validation(format!(
                "campaign {} has not been ingested",
                self.id
            )));
        }

        // TODO: event_id uses Uuid::new_v4() which breaks replay determinism.
        // See TODO on `ingest_campaign()` for details.
        let event = ContentEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: CAMPAIGN_VALIDATED_EVENT_TYPE.to_owned(),
                aggregate_id: self.id,
                sequence_number: self.next_sequence_number(),
                correlation_id,
                causation_id: correlation_id,
                occurred_at: clock.now(),
            },
            kind: ContentEventKind::CampaignValidated(CampaignValidated {
                campaign_id: self.id,
            }),
        };

        self.uncommitted_events.push(event);
        Ok(())
    }

    /// Compiles a campaign into runtime format, producing a `CampaignCompiled` event.
    ///
    /// TODO: Add actual compilation logic — transform validated campaign source
    /// into optimized runtime format (scene graph, lookup tables, etc.).
    ///
    /// # Errors
    ///
    /// Returns `DomainError::Validation` if the campaign has not been validated.
    pub fn compile_campaign(
        &mut self,
        correlation_id: Uuid,
        clock: &dyn Clock,
    ) -> Result<(), DomainError> {
        if !self.validated {
            return Err(DomainError::Validation(format!(
                "campaign {} has not been validated",
                self.id
            )));
        }

        let version_hash = self.version_hash.clone().ok_or_else(|| {
            DomainError::Infrastructure(format!(
                "campaign {} is validated but has no version_hash — invariant violated",
                self.id
            ))
        })?;

        // TODO: event_id uses Uuid::new_v4() which breaks replay determinism.
        // See TODO on `ingest_campaign()` for details.
        let event = ContentEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: CAMPAIGN_COMPILED_EVENT_TYPE.to_owned(),
                aggregate_id: self.id,
                sequence_number: self.next_sequence_number(),
                correlation_id,
                causation_id: correlation_id,
                occurred_at: clock.now(),
            },
            kind: ContentEventKind::CampaignCompiled(CampaignCompiled {
                campaign_id: self.id,
                version_hash,
            }),
        };

        self.uncommitted_events.push(event);
        Ok(())
    }
}

impl AggregateRoot for Campaign {
    type Event = ContentEvent;

    fn aggregate_id(&self) -> Uuid {
        self.id
    }

    fn version(&self) -> i64 {
        self.version
    }

    fn apply(&mut self, event: &Self::Event) {
        match &event.kind {
            ContentEventKind::CampaignIngested(payload) => {
                self.ingested = true;
                self.version_hash = Some(payload.version_hash.clone());
            }
            ContentEventKind::CampaignValidated(_) => {
                self.validated = true;
            }
            ContentEventKind::CampaignCompiled(_) => {}
        }
        self.version += 1;
    }

    fn uncommitted_events(&self) -> &[Self::Event] {
        &self.uncommitted_events
    }

    fn clear_uncommitted_events(&mut self) {
        self.uncommitted_events.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use otherworlds_core::aggregate::AggregateRoot;
    use otherworlds_core::event::DomainEvent;
    use otherworlds_test_support::FixedClock;

    #[test]
    fn test_ingest_campaign_produces_campaign_ingested_event() {
        // Arrange
        let campaign_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut campaign = Campaign::new(campaign_id);

        // Act
        campaign
            .ingest_campaign("# My Campaign", correlation_id, &clock)
            .unwrap();

        // Assert
        let events = campaign.uncommitted_events();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.event_type(), CAMPAIGN_INGESTED_EVENT_TYPE);

        let meta = event.metadata();
        assert_eq!(meta.aggregate_id, campaign_id);
        assert_eq!(meta.sequence_number, 1);
        assert_eq!(meta.correlation_id, correlation_id);
        assert_eq!(meta.causation_id, correlation_id);
        assert_eq!(meta.occurred_at, fixed_now);

        match &event.kind {
            ContentEventKind::CampaignIngested(payload) => {
                assert_eq!(payload.campaign_id, campaign_id);
                assert!(!payload.version_hash.is_empty());
            }
            other => panic!("expected CampaignIngested, got {other:?}"),
        }
    }

    #[test]
    fn test_ingest_campaign_returns_error_when_already_ingested() {
        // Arrange
        let campaign_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut campaign = Campaign::new(campaign_id);

        campaign
            .ingest_campaign("# My Campaign", Uuid::new_v4(), &clock)
            .unwrap();
        for event in campaign.uncommitted_events().to_vec() {
            campaign.apply(&event);
        }
        campaign.clear_uncommitted_events();

        // Act
        let result = campaign.ingest_campaign("# My Campaign", Uuid::new_v4(), &clock);

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert!(msg.contains(&campaign_id.to_string()));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn test_validate_campaign_produces_campaign_validated_event() {
        // Arrange
        let campaign_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut campaign = Campaign::new(campaign_id);

        // Ingest first.
        campaign
            .ingest_campaign("# My Campaign", Uuid::new_v4(), &clock)
            .unwrap();
        for event in campaign.uncommitted_events().to_vec() {
            campaign.apply(&event);
        }
        campaign.clear_uncommitted_events();

        // Act
        campaign.validate_campaign(correlation_id, &clock).unwrap();

        // Assert
        let events = campaign.uncommitted_events();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.event_type(), CAMPAIGN_VALIDATED_EVENT_TYPE);

        let meta = event.metadata();
        assert_eq!(meta.aggregate_id, campaign_id);
        assert_eq!(meta.sequence_number, 2);
        assert_eq!(meta.correlation_id, correlation_id);
        assert_eq!(meta.causation_id, correlation_id);
        assert_eq!(meta.occurred_at, fixed_now);

        match &event.kind {
            ContentEventKind::CampaignValidated(payload) => {
                assert_eq!(payload.campaign_id, campaign_id);
            }
            other => panic!("expected CampaignValidated, got {other:?}"),
        }
    }

    #[test]
    fn test_validate_campaign_returns_error_when_not_ingested() {
        // Arrange
        let campaign_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut campaign = Campaign::new(campaign_id);

        // Act
        let result = campaign.validate_campaign(Uuid::new_v4(), &clock);

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert!(msg.contains(&campaign_id.to_string()));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn test_compile_campaign_produces_campaign_compiled_event() {
        // Arrange
        let campaign_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut campaign = Campaign::new(campaign_id);

        // Ingest then validate.
        campaign
            .ingest_campaign("# My Campaign", Uuid::new_v4(), &clock)
            .unwrap();
        for event in campaign.uncommitted_events().to_vec() {
            campaign.apply(&event);
        }
        campaign.clear_uncommitted_events();

        campaign.validate_campaign(Uuid::new_v4(), &clock).unwrap();
        for event in campaign.uncommitted_events().to_vec() {
            campaign.apply(&event);
        }
        campaign.clear_uncommitted_events();

        // Act
        campaign.compile_campaign(correlation_id, &clock).unwrap();

        // Assert
        let events = campaign.uncommitted_events();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.event_type(), CAMPAIGN_COMPILED_EVENT_TYPE);

        let meta = event.metadata();
        assert_eq!(meta.aggregate_id, campaign_id);
        assert_eq!(meta.sequence_number, 3);
        assert_eq!(meta.correlation_id, correlation_id);
        assert_eq!(meta.causation_id, correlation_id);
        assert_eq!(meta.occurred_at, fixed_now);

        match &event.kind {
            ContentEventKind::CampaignCompiled(payload) => {
                assert_eq!(payload.campaign_id, campaign_id);
                assert!(!payload.version_hash.is_empty());
            }
            other => panic!("expected CampaignCompiled, got {other:?}"),
        }
    }

    #[test]
    fn test_compile_campaign_returns_error_when_not_validated() {
        // Arrange — campaign is ingested but not validated.
        let campaign_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut campaign = Campaign::new(campaign_id);

        campaign
            .ingest_campaign("# My Campaign", Uuid::new_v4(), &clock)
            .unwrap();
        for event in campaign.uncommitted_events().to_vec() {
            campaign.apply(&event);
        }
        campaign.clear_uncommitted_events();

        // Act
        let result = campaign.compile_campaign(Uuid::new_v4(), &clock);

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert!(msg.contains(&campaign_id.to_string()));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn test_ingest_campaign_returns_error_when_source_is_empty() {
        // Arrange
        let campaign_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut campaign = Campaign::new(campaign_id);

        // Act
        let result = campaign.ingest_campaign("", Uuid::new_v4(), &clock);

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert!(msg.contains("empty"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn test_version_hash_is_stable_sha256() {
        // SHA-256 of "# My Campaign" must be stable across Rust versions
        // and platforms. This known value was computed via `echo -n "# My Campaign" | shasum -a 256`.
        let hash = Campaign::compute_version_hash("# My Campaign");

        assert_eq!(
            hash,
            "bf576a9cb4584e476d0195b21ef1c5ba67573544ad3870920911aefed42e4798"
        );
    }
}
