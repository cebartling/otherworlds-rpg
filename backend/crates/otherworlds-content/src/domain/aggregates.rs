//! Aggregate roots for the Content Authoring context.

use sha2::{Digest, Sha256};

use otherworlds_core::aggregate::AggregateRoot;
use otherworlds_core::clock::Clock;
use otherworlds_core::error::DomainError;
use otherworlds_core::event::EventMetadata;
use otherworlds_core::rng::DeterministicRng;
use uuid::Uuid;

use super::compiler::compile_parsed_campaign;
use super::parser::parse_campaign;
use super::validator::validate_parsed_campaign;

use super::events::{
    CAMPAIGN_ARCHIVED_EVENT_TYPE, CAMPAIGN_COMPILED_EVENT_TYPE, CAMPAIGN_INGESTED_EVENT_TYPE,
    CAMPAIGN_VALIDATED_EVENT_TYPE, CampaignArchived, CampaignCompiled, CampaignIngested,
    CampaignValidated, ContentEvent, ContentEventKind,
};

/// The aggregate root for a campaign.
#[derive(Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct Campaign {
    /// Aggregate identifier.
    pub id: Uuid,
    /// Current version (event count).
    pub(crate) version: i64,
    /// Whether the campaign has been ingested.
    pub(crate) ingested: bool,
    /// Whether the campaign has been validated.
    pub(crate) validated: bool,
    /// Whether the campaign has been compiled into runtime format.
    pub(crate) compiled: bool,
    /// Whether the campaign has been archived (soft-deleted).
    pub(crate) archived: bool,
    /// The campaign version hash (set on ingestion).
    pub(crate) version_hash: Option<String>,
    /// The raw source content (set on ingestion).
    pub(crate) source: Option<String>,
    /// The compiled campaign data as JSON (set on compilation).
    pub(crate) compiled_data: Option<String>,
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
            compiled: false,
            archived: false,
            version_hash: None,
            source: None,
            compiled_data: None,
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
        rng: &mut dyn DeterministicRng,
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

        let event = ContentEvent {
            metadata: EventMetadata {
                event_id: rng.next_uuid(),
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
                source: source.to_owned(),
            }),
        };

        self.uncommitted_events.push(event);
        Ok(())
    }

    /// Validates a campaign, producing a `CampaignValidated` event.
    ///
    /// Parses the campaign source Markdown and validates structural
    /// integrity (scene references, NPC references, front-matter).
    ///
    /// # Errors
    ///
    /// Returns `DomainError::Validation` if the campaign has not been ingested
    /// or if the campaign source fails structural validation.
    pub fn validate_campaign(
        &mut self,
        correlation_id: Uuid,
        clock: &dyn Clock,
        rng: &mut dyn DeterministicRng,
    ) -> Result<(), DomainError> {
        if !self.ingested {
            return Err(DomainError::Validation(format!(
                "campaign {} has not been ingested",
                self.id
            )));
        }

        let source = self.source.as_deref().ok_or_else(|| {
            DomainError::Infrastructure(format!(
                "campaign {} is ingested but has no source — invariant violated",
                self.id
            ))
        })?;

        let parsed = parse_campaign(source)?;
        validate_parsed_campaign(&parsed)?;

        let event = ContentEvent {
            metadata: EventMetadata {
                event_id: rng.next_uuid(),
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
    /// Parses the campaign source and compiles it into an indexed
    /// `CompiledCampaign` structure serialized as JSON.
    ///
    /// # Errors
    ///
    /// Returns `DomainError::Validation` if the campaign has not been validated.
    /// Returns `DomainError::Infrastructure` if invariants are violated.
    pub fn compile_campaign(
        &mut self,
        correlation_id: Uuid,
        clock: &dyn Clock,
        rng: &mut dyn DeterministicRng,
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

        let source = self.source.as_deref().ok_or_else(|| {
            DomainError::Infrastructure(format!(
                "campaign {} is validated but has no source — invariant violated",
                self.id
            ))
        })?;

        let parsed = parse_campaign(source)?;
        let compiled = compile_parsed_campaign(&parsed);
        let compiled_data = serde_json::to_string(&compiled).map_err(|e| {
            DomainError::Infrastructure(format!("compiled campaign serialization failed: {e}"))
        })?;

        let event = ContentEvent {
            metadata: EventMetadata {
                event_id: rng.next_uuid(),
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
                compiled_data,
            }),
        };

        self.uncommitted_events.push(event);
        Ok(())
    }

    /// Archives the campaign (soft-delete), producing a `CampaignArchived` event.
    ///
    /// # Errors
    ///
    /// Returns `DomainError::Validation` if the campaign is already archived.
    pub fn archive(
        &mut self,
        correlation_id: Uuid,
        clock: &dyn Clock,
        rng: &mut dyn DeterministicRng,
    ) -> Result<(), DomainError> {
        if self.archived {
            return Err(DomainError::Validation(
                "campaign is already archived".into(),
            ));
        }

        let event = ContentEvent {
            metadata: EventMetadata {
                event_id: rng.next_uuid(),
                event_type: CAMPAIGN_ARCHIVED_EVENT_TYPE.to_owned(),
                aggregate_id: self.id,
                sequence_number: self.next_sequence_number(),
                correlation_id,
                causation_id: correlation_id,
                occurred_at: clock.now(),
            },
            kind: ContentEventKind::CampaignArchived(CampaignArchived {
                campaign_id: self.id,
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
                self.source = Some(payload.source.clone());
            }
            ContentEventKind::CampaignValidated(_) => {
                self.validated = true;
            }
            ContentEventKind::CampaignCompiled(payload) => {
                self.compiled = true;
                self.compiled_data = Some(payload.compiled_data.clone());
            }
            ContentEventKind::CampaignArchived(_) => {
                self.archived = true;
            }
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
    use otherworlds_test_support::{FixedClock, MockRng};

    /// Valid campaign source with YAML front-matter and one scene.
    const VALID_CAMPAIGN_SOURCE: &str =
        "---\ntitle: \"Test Campaign\"\n---\n\n# Scene: start\n\nHello world.\n";

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
            .ingest_campaign("# My Campaign", correlation_id, &clock, &mut MockRng)
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
                assert_eq!(payload.source, "# My Campaign");
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
            .ingest_campaign("# My Campaign", Uuid::new_v4(), &clock, &mut MockRng)
            .unwrap();
        for event in campaign.uncommitted_events().to_vec() {
            campaign.apply(&event);
        }
        campaign.clear_uncommitted_events();

        // Act
        let result =
            campaign.ingest_campaign("# My Campaign", Uuid::new_v4(), &clock, &mut MockRng);

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

        // Ingest first (with valid source for validation).
        campaign
            .ingest_campaign(VALID_CAMPAIGN_SOURCE, Uuid::new_v4(), &clock, &mut MockRng)
            .unwrap();
        for event in campaign.uncommitted_events().to_vec() {
            campaign.apply(&event);
        }
        campaign.clear_uncommitted_events();

        // Act
        campaign
            .validate_campaign(correlation_id, &clock, &mut MockRng)
            .unwrap();

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
        let result = campaign.validate_campaign(Uuid::new_v4(), &clock, &mut MockRng);

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
    fn test_validate_campaign_returns_error_for_invalid_source() {
        // Arrange — ingest source with no front-matter (invalid for validation).
        let campaign_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut campaign = Campaign::new(campaign_id);

        campaign
            .ingest_campaign("# My Campaign", Uuid::new_v4(), &clock, &mut MockRng)
            .unwrap();
        for event in campaign.uncommitted_events().to_vec() {
            campaign.apply(&event);
        }
        campaign.clear_uncommitted_events();

        // Act
        let result = campaign.validate_campaign(Uuid::new_v4(), &clock, &mut MockRng);

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert!(msg.contains("front-matter"));
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

        // Ingest then validate with valid source.
        campaign
            .ingest_campaign(VALID_CAMPAIGN_SOURCE, Uuid::new_v4(), &clock, &mut MockRng)
            .unwrap();
        for event in campaign.uncommitted_events().to_vec() {
            campaign.apply(&event);
        }
        campaign.clear_uncommitted_events();

        campaign
            .validate_campaign(Uuid::new_v4(), &clock, &mut MockRng)
            .unwrap();
        for event in campaign.uncommitted_events().to_vec() {
            campaign.apply(&event);
        }
        campaign.clear_uncommitted_events();

        // Act
        campaign
            .compile_campaign(correlation_id, &clock, &mut MockRng)
            .unwrap();

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
                assert!(!payload.compiled_data.is_empty());
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
            .ingest_campaign(VALID_CAMPAIGN_SOURCE, Uuid::new_v4(), &clock, &mut MockRng)
            .unwrap();
        for event in campaign.uncommitted_events().to_vec() {
            campaign.apply(&event);
        }
        campaign.clear_uncommitted_events();

        // Act
        let result = campaign.compile_campaign(Uuid::new_v4(), &clock, &mut MockRng);

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
        let result = campaign.ingest_campaign("", Uuid::new_v4(), &clock, &mut MockRng);

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
    fn test_archive_produces_campaign_archived_event() {
        // Arrange
        let campaign_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut campaign = Campaign::new(campaign_id);

        // Act
        campaign
            .archive(correlation_id, &clock, &mut MockRng)
            .unwrap();

        // Assert
        let events = campaign.uncommitted_events();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.event_type(), CAMPAIGN_ARCHIVED_EVENT_TYPE);

        let meta = event.metadata();
        assert_eq!(meta.aggregate_id, campaign_id);
        assert_eq!(meta.sequence_number, 1);
        assert_eq!(meta.correlation_id, correlation_id);
        assert_eq!(meta.causation_id, correlation_id);
        assert_eq!(meta.occurred_at, fixed_now);

        match &event.kind {
            ContentEventKind::CampaignArchived(payload) => {
                assert_eq!(payload.campaign_id, campaign_id);
            }
            other => panic!("expected CampaignArchived, got {other:?}"),
        }
    }

    #[test]
    fn test_archive_returns_error_when_already_archived() {
        // Arrange
        let campaign_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut campaign = Campaign::new(campaign_id);

        campaign
            .archive(Uuid::new_v4(), &clock, &mut MockRng)
            .unwrap();
        for event in campaign.uncommitted_events().to_vec() {
            campaign.apply(&event);
        }
        campaign.clear_uncommitted_events();

        // Act
        let result = campaign.archive(Uuid::new_v4(), &clock, &mut MockRng);

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert!(msg.contains("already archived"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn test_apply_campaign_archived_sets_archived_flag() {
        // Arrange
        let campaign_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let mut campaign = Campaign::new(campaign_id);
        let event = ContentEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: CAMPAIGN_ARCHIVED_EVENT_TYPE.to_owned(),
                aggregate_id: campaign_id,
                sequence_number: 1,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
            kind: ContentEventKind::CampaignArchived(CampaignArchived { campaign_id }),
        };

        // Act
        campaign.apply(&event);

        // Assert
        assert!(campaign.archived);
        assert_eq!(campaign.version, 1);
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

    #[test]
    fn test_apply_campaign_ingested_sets_source() {
        // Arrange
        let campaign_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let mut campaign = Campaign::new(campaign_id);
        let event = ContentEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: CAMPAIGN_INGESTED_EVENT_TYPE.to_owned(),
                aggregate_id: campaign_id,
                sequence_number: 1,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
            kind: ContentEventKind::CampaignIngested(CampaignIngested {
                campaign_id,
                version_hash: "abc123".to_owned(),
                source: "# My Campaign".to_owned(),
            }),
        };

        // Act
        campaign.apply(&event);

        // Assert
        assert_eq!(campaign.source, Some("# My Campaign".to_owned()));
        assert!(campaign.ingested);
        assert_eq!(campaign.version, 1);
    }

    #[test]
    fn test_apply_campaign_compiled_sets_compiled_flag_and_data() {
        // Arrange
        let campaign_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let mut campaign = Campaign::new(campaign_id);

        // Pre-apply ingested so the aggregate is in a valid state.
        let ingested_event = ContentEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: CAMPAIGN_INGESTED_EVENT_TYPE.to_owned(),
                aggregate_id: campaign_id,
                sequence_number: 1,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
            kind: ContentEventKind::CampaignIngested(CampaignIngested {
                campaign_id,
                version_hash: "abc123".to_owned(),
                source: "# My Campaign".to_owned(),
            }),
        };
        campaign.apply(&ingested_event);

        let compiled_event = ContentEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: CAMPAIGN_COMPILED_EVENT_TYPE.to_owned(),
                aggregate_id: campaign_id,
                sequence_number: 3,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
            kind: ContentEventKind::CampaignCompiled(CampaignCompiled {
                campaign_id,
                version_hash: "abc123".to_owned(),
                compiled_data: "{\"title\":\"Test\"}".to_owned(),
            }),
        };

        // Act
        campaign.apply(&compiled_event);

        // Assert
        assert!(campaign.compiled);
        assert_eq!(
            campaign.compiled_data,
            Some("{\"title\":\"Test\"}".to_owned())
        );
        assert_eq!(campaign.version, 2);
    }

    #[test]
    fn test_compile_campaign_sets_compiled_flag_after_apply() {
        // Arrange — full lifecycle: ingest → validate → compile.
        let campaign_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut campaign = Campaign::new(campaign_id);

        // Ingest with valid source.
        campaign
            .ingest_campaign(VALID_CAMPAIGN_SOURCE, Uuid::new_v4(), &clock, &mut MockRng)
            .unwrap();
        for event in campaign.uncommitted_events().to_vec() {
            campaign.apply(&event);
        }
        campaign.clear_uncommitted_events();

        // Validate.
        campaign
            .validate_campaign(Uuid::new_v4(), &clock, &mut MockRng)
            .unwrap();
        for event in campaign.uncommitted_events().to_vec() {
            campaign.apply(&event);
        }
        campaign.clear_uncommitted_events();

        // Compile.
        campaign
            .compile_campaign(Uuid::new_v4(), &clock, &mut MockRng)
            .unwrap();
        for event in campaign.uncommitted_events().to_vec() {
            campaign.apply(&event);
        }
        campaign.clear_uncommitted_events();

        // Assert
        assert!(campaign.compiled);
        assert!(campaign.ingested);
        assert!(campaign.validated);
        assert!(campaign.compiled_data.is_some());
    }
}
