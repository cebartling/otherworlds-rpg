//! Aggregate roots for the Narrative Orchestration context.

use otherworlds_core::aggregate::AggregateRoot;
use otherworlds_core::clock::Clock;
use otherworlds_core::error::DomainError;
use otherworlds_core::event::EventMetadata;
use otherworlds_core::rng::DeterministicRng;
use uuid::Uuid;

use super::events::{
    BeatAdvanced, ChoicePresented, ChoiceSelected, NarrativeEvent, NarrativeEventKind,
    SceneStarted, SessionArchived,
};
use super::value_objects::{ChoiceOption, SceneData};

/// The aggregate root for a narrative session.
#[derive(Debug)]
pub struct NarrativeSession {
    /// Aggregate identifier.
    pub id: Uuid,
    /// Current version (event count).
    pub(crate) version: i64,
    /// The most recent beat ID.
    pub(crate) current_beat_id: Option<Uuid>,
    /// All choice IDs presented in this session.
    pub(crate) choice_ids: Vec<Uuid>,
    /// The current scene ID (author-defined).
    pub(crate) current_scene_id: Option<String>,
    /// History of scene IDs visited in order.
    pub(crate) scene_history: Vec<String>,
    /// Active choice options for the current scene.
    pub(crate) active_choice_options: Vec<ChoiceOption>,
    /// Whether this session has been archived (soft-deleted).
    pub(crate) archived: bool,
    /// Uncommitted events pending persistence.
    uncommitted_events: Vec<NarrativeEvent>,
}

impl NarrativeSession {
    /// Creates a new narrative session.
    #[must_use]
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            version: 0,
            current_beat_id: None,
            choice_ids: Vec::new(),
            current_scene_id: None,
            scene_history: Vec::new(),
            active_choice_options: Vec::new(),
            archived: false,
            uncommitted_events: Vec::new(),
        }
    }

    /// Returns the next sequence number for a new event.
    #[allow(clippy::cast_possible_wrap)]
    fn next_sequence_number(&self) -> i64 {
        self.version + self.uncommitted_events.len() as i64 + 1
    }

    /// Advances the narrative to the next beat, producing a `BeatAdvanced` event.
    pub fn advance_beat(
        &mut self,
        correlation_id: Uuid,
        clock: &dyn Clock,
        rng: &mut dyn DeterministicRng,
    ) {
        let event = NarrativeEvent {
            metadata: EventMetadata {
                event_id: rng.next_uuid(),
                event_type: "narrative.beat_advanced".to_owned(),
                aggregate_id: self.id,
                sequence_number: self.next_sequence_number(),
                correlation_id,
                causation_id: correlation_id,
                occurred_at: clock.now(),
            },
            kind: NarrativeEventKind::BeatAdvanced(BeatAdvanced {
                session_id: self.id,
                beat_id: rng.next_uuid(),
            }),
        };

        self.uncommitted_events.push(event);
    }

    /// Presents a choice to the player, producing a `ChoicePresented` event.
    pub fn present_choice(
        &mut self,
        correlation_id: Uuid,
        clock: &dyn Clock,
        rng: &mut dyn DeterministicRng,
    ) {
        let event = NarrativeEvent {
            metadata: EventMetadata {
                event_id: rng.next_uuid(),
                event_type: "narrative.choice_presented".to_owned(),
                aggregate_id: self.id,
                sequence_number: self.next_sequence_number(),
                correlation_id,
                causation_id: correlation_id,
                occurred_at: clock.now(),
            },
            kind: NarrativeEventKind::ChoicePresented(ChoicePresented {
                session_id: self.id,
                choice_id: rng.next_uuid(),
            }),
        };

        self.uncommitted_events.push(event);
    }

    /// Enters a scene, producing a `SceneStarted` event.
    ///
    /// # Errors
    ///
    /// Returns `DomainError::Validation` if the session is archived.
    pub fn enter_scene(
        &mut self,
        scene_data: &SceneData,
        correlation_id: Uuid,
        clock: &dyn Clock,
        rng: &mut dyn DeterministicRng,
    ) -> Result<(), DomainError> {
        if self.archived {
            return Err(DomainError::Validation("session is archived".into()));
        }

        let event = NarrativeEvent {
            metadata: EventMetadata {
                event_id: rng.next_uuid(),
                event_type: "narrative.scene_started".to_owned(),
                aggregate_id: self.id,
                sequence_number: self.next_sequence_number(),
                correlation_id,
                causation_id: correlation_id,
                occurred_at: clock.now(),
            },
            kind: NarrativeEventKind::SceneStarted(SceneStarted {
                session_id: self.id,
                scene_id: scene_data.scene_id.clone(),
                narrative_text: scene_data.narrative_text.clone(),
                choices: scene_data.choices.clone(),
                npc_refs: scene_data.npc_refs.clone(),
            }),
        };

        self.uncommitted_events.push(event);
        Ok(())
    }

    /// Selects a choice, producing a `ChoiceSelected` event followed by a
    /// `SceneStarted` event for the target scene.
    ///
    /// # Errors
    ///
    /// Returns `DomainError::Validation` if:
    /// - The session is archived
    /// - No scene is active
    /// - The choice index is out of bounds
    pub fn select_choice(
        &mut self,
        choice_index: usize,
        target_scene_data: &SceneData,
        correlation_id: Uuid,
        clock: &dyn Clock,
        rng: &mut dyn DeterministicRng,
    ) -> Result<(), DomainError> {
        if self.archived {
            return Err(DomainError::Validation("session is archived".into()));
        }

        let current_scene_id = self
            .current_scene_id
            .as_ref()
            .ok_or_else(|| DomainError::Validation("no active scene".into()))?;

        let choice = self
            .active_choice_options
            .get(choice_index)
            .ok_or_else(|| {
                DomainError::Validation(format!(
                    "choice index {choice_index} out of bounds (available: {})",
                    self.active_choice_options.len()
                ))
            })?;

        let choice_selected_event = NarrativeEvent {
            metadata: EventMetadata {
                event_id: rng.next_uuid(),
                event_type: "narrative.choice_selected".to_owned(),
                aggregate_id: self.id,
                sequence_number: self.next_sequence_number(),
                correlation_id,
                causation_id: correlation_id,
                occurred_at: clock.now(),
            },
            kind: NarrativeEventKind::ChoiceSelected(ChoiceSelected {
                session_id: self.id,
                choice_label: choice.label.clone(),
                from_scene_id: current_scene_id.clone(),
                to_scene_id: choice.target_scene_id.clone(),
            }),
        };
        self.uncommitted_events.push(choice_selected_event);

        let scene_started_event = NarrativeEvent {
            metadata: EventMetadata {
                event_id: rng.next_uuid(),
                event_type: "narrative.scene_started".to_owned(),
                aggregate_id: self.id,
                sequence_number: self.next_sequence_number(),
                correlation_id,
                causation_id: correlation_id,
                occurred_at: clock.now(),
            },
            kind: NarrativeEventKind::SceneStarted(SceneStarted {
                session_id: self.id,
                scene_id: target_scene_data.scene_id.clone(),
                narrative_text: target_scene_data.narrative_text.clone(),
                choices: target_scene_data.choices.clone(),
                npc_refs: target_scene_data.npc_refs.clone(),
            }),
        };
        self.uncommitted_events.push(scene_started_event);

        Ok(())
    }

    /// Archives (soft-deletes) a session, producing a `SessionArchived` event.
    ///
    /// # Errors
    ///
    /// Returns `DomainError::Validation` if the session is already archived.
    pub fn archive(
        &mut self,
        correlation_id: Uuid,
        clock: &dyn Clock,
        rng: &mut dyn DeterministicRng,
    ) -> Result<(), DomainError> {
        if self.archived {
            return Err(DomainError::Validation(
                "session is already archived".into(),
            ));
        }

        let event = NarrativeEvent {
            metadata: EventMetadata {
                event_id: rng.next_uuid(),
                event_type: "narrative.session_archived".to_owned(),
                aggregate_id: self.id,
                sequence_number: self.next_sequence_number(),
                correlation_id,
                causation_id: correlation_id,
                occurred_at: clock.now(),
            },
            kind: NarrativeEventKind::SessionArchived(SessionArchived {
                session_id: self.id,
            }),
        };

        self.uncommitted_events.push(event);
        Ok(())
    }
}

impl AggregateRoot for NarrativeSession {
    type Event = NarrativeEvent;

    fn aggregate_id(&self) -> Uuid {
        self.id
    }

    fn version(&self) -> i64 {
        self.version
    }

    fn apply(&mut self, event: &Self::Event) {
        match &event.kind {
            NarrativeEventKind::BeatAdvanced(payload) => {
                self.current_beat_id = Some(payload.beat_id);
            }
            NarrativeEventKind::ChoicePresented(payload) => {
                self.choice_ids.push(payload.choice_id);
            }
            NarrativeEventKind::SceneStarted(payload) => {
                self.current_scene_id = Some(payload.scene_id.clone());
                self.scene_history.push(payload.scene_id.clone());
                self.active_choice_options.clone_from(&payload.choices);
            }
            NarrativeEventKind::ChoiceSelected(_) => {
                self.active_choice_options.clear();
            }
            NarrativeEventKind::SessionArchived(_) => {
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

    #[test]
    fn test_advance_beat_produces_beat_advanced_event() {
        // Arrange
        let session_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut session = NarrativeSession::new(session_id);
        let mut rng = MockRng;

        // Act
        session.advance_beat(correlation_id, &clock, &mut rng);

        // Assert
        let events = session.uncommitted_events();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.event_type(), "narrative.beat_advanced");

        let meta = event.metadata();
        assert_eq!(meta.aggregate_id, session_id);
        assert_eq!(meta.sequence_number, 1);
        assert_eq!(meta.correlation_id, correlation_id);
        assert_eq!(meta.causation_id, correlation_id);
        assert_eq!(meta.occurred_at, fixed_now);

        match &event.kind {
            NarrativeEventKind::BeatAdvanced(payload) => {
                assert_eq!(payload.session_id, session_id);
            }
            other => panic!("expected BeatAdvanced, got {other:?}"),
        }
    }

    #[test]
    fn test_apply_beat_advanced_sets_current_beat_id() {
        // Arrange
        let session_id = Uuid::new_v4();
        let beat_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let mut session = NarrativeSession::new(session_id);
        let event = NarrativeEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "narrative.beat_advanced".to_owned(),
                aggregate_id: session_id,
                sequence_number: 1,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
            kind: NarrativeEventKind::BeatAdvanced(BeatAdvanced {
                session_id,
                beat_id,
            }),
        };

        // Act
        session.apply(&event);

        // Assert
        assert_eq!(session.current_beat_id, Some(beat_id));
        assert_eq!(session.version, 1);
    }

    #[test]
    fn test_apply_choice_presented_pushes_to_choice_ids() {
        // Arrange
        let session_id = Uuid::new_v4();
        let choice_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let mut session = NarrativeSession::new(session_id);
        let event = NarrativeEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "narrative.choice_presented".to_owned(),
                aggregate_id: session_id,
                sequence_number: 1,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
            kind: NarrativeEventKind::ChoicePresented(ChoicePresented {
                session_id,
                choice_id,
            }),
        };

        // Act
        session.apply(&event);

        // Assert
        assert_eq!(session.choice_ids, vec![choice_id]);
        assert_eq!(session.version, 1);
    }

    #[test]
    fn test_present_choice_produces_choice_presented_event() {
        // Arrange
        let session_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut session = NarrativeSession::new(session_id);
        let mut rng = MockRng;

        // Act
        session.present_choice(correlation_id, &clock, &mut rng);

        // Assert
        let events = session.uncommitted_events();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.event_type(), "narrative.choice_presented");

        let meta = event.metadata();
        assert_eq!(meta.aggregate_id, session_id);
        assert_eq!(meta.sequence_number, 1);
        assert_eq!(meta.correlation_id, correlation_id);
        assert_eq!(meta.causation_id, correlation_id);
        assert_eq!(meta.occurred_at, fixed_now);

        match &event.kind {
            NarrativeEventKind::ChoicePresented(payload) => {
                assert_eq!(payload.session_id, session_id);
            }
            other => panic!("expected ChoicePresented, got {other:?}"),
        }
    }

    #[test]
    fn test_archive_produces_session_archived_event() {
        // Arrange
        let session_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut session = NarrativeSession::new(session_id);
        let mut rng = MockRng;

        // Act
        let result = session.archive(correlation_id, &clock, &mut rng);

        // Assert
        assert!(result.is_ok());

        let events = session.uncommitted_events();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.event_type(), "narrative.session_archived");

        let meta = event.metadata();
        assert_eq!(meta.aggregate_id, session_id);
        assert_eq!(meta.sequence_number, 1);
        assert_eq!(meta.correlation_id, correlation_id);
        assert_eq!(meta.causation_id, correlation_id);
        assert_eq!(meta.occurred_at, fixed_now);

        match &event.kind {
            NarrativeEventKind::SessionArchived(payload) => {
                assert_eq!(payload.session_id, session_id);
            }
            other => panic!("expected SessionArchived, got {other:?}"),
        }
    }

    #[test]
    fn test_apply_session_archived_sets_flag() {
        // Arrange
        let session_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let mut session = NarrativeSession::new(session_id);
        let event = NarrativeEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "narrative.session_archived".to_owned(),
                aggregate_id: session_id,
                sequence_number: 1,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
            kind: NarrativeEventKind::SessionArchived(SessionArchived { session_id }),
        };

        // Act
        session.apply(&event);

        // Assert
        assert!(session.archived);
        assert_eq!(session.version, 1);
    }

    #[test]
    fn test_archive_already_archived_returns_error() {
        // Arrange
        let session_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut session = NarrativeSession::new(session_id);
        session.archived = true;
        let mut rng = MockRng;

        // Act
        let result = session.archive(correlation_id, &clock, &mut rng);

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert_eq!(msg, "session is already archived");
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    fn sample_scene_data(scene_id: &str, choices: Vec<(&str, &str)>) -> SceneData {
        SceneData {
            scene_id: scene_id.to_owned(),
            narrative_text: format!("You are in {scene_id}."),
            choices: choices
                .into_iter()
                .map(|(label, target)| ChoiceOption {
                    label: label.to_owned(),
                    target_scene_id: target.to_owned(),
                })
                .collect(),
            npc_refs: vec![],
        }
    }

    #[test]
    fn test_enter_scene_produces_scene_started_event() {
        // Arrange
        let session_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut session = NarrativeSession::new(session_id);
        let mut rng = MockRng;
        let scene = sample_scene_data("tavern", vec![("Leave", "street")]);

        // Act
        let result = session.enter_scene(&scene, correlation_id, &clock, &mut rng);

        // Assert
        assert!(result.is_ok());
        let events = session.uncommitted_events();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.event_type(), "narrative.scene_started");

        match &event.kind {
            NarrativeEventKind::SceneStarted(payload) => {
                assert_eq!(payload.session_id, session_id);
                assert_eq!(payload.scene_id, "tavern");
                assert_eq!(payload.narrative_text, "You are in tavern.");
                assert_eq!(payload.choices.len(), 1);
                assert_eq!(payload.choices[0].label, "Leave");
                assert_eq!(payload.choices[0].target_scene_id, "street");
            }
            other => panic!("expected SceneStarted, got {other:?}"),
        }
    }

    #[test]
    fn test_enter_scene_rejects_archived_session() {
        // Arrange
        let session_id = Uuid::new_v4();
        let mut session = NarrativeSession::new(session_id);
        session.archived = true;
        let clock = FixedClock(Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap());
        let mut rng = MockRng;
        let scene = sample_scene_data("tavern", vec![]);

        // Act
        let result = session.enter_scene(&scene, Uuid::new_v4(), &clock, &mut rng);

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => assert_eq!(msg, "session is archived"),
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn test_apply_scene_started_sets_scene_state() {
        // Arrange
        let session_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let mut session = NarrativeSession::new(session_id);
        let choices = vec![ChoiceOption {
            label: "Go north".to_owned(),
            target_scene_id: "forest".to_owned(),
        }];
        let event = NarrativeEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "narrative.scene_started".to_owned(),
                aggregate_id: session_id,
                sequence_number: 1,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
            kind: NarrativeEventKind::SceneStarted(SceneStarted {
                session_id,
                scene_id: "start".to_owned(),
                narrative_text: "You begin.".to_owned(),
                choices: choices.clone(),
                npc_refs: vec![],
            }),
        };

        // Act
        session.apply(&event);

        // Assert
        assert_eq!(session.current_scene_id, Some("start".to_owned()));
        assert_eq!(session.scene_history, vec!["start"]);
        assert_eq!(session.active_choice_options, choices);
        assert_eq!(session.version, 1);
    }

    #[test]
    fn test_apply_choice_selected_clears_active_choices() {
        // Arrange
        let session_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let mut session = NarrativeSession::new(session_id);
        session.active_choice_options = vec![ChoiceOption {
            label: "Go".to_owned(),
            target_scene_id: "next".to_owned(),
        }];
        session.current_scene_id = Some("start".to_owned());

        let event = NarrativeEvent {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "narrative.choice_selected".to_owned(),
                aggregate_id: session_id,
                sequence_number: 1,
                correlation_id: Uuid::new_v4(),
                causation_id: Uuid::new_v4(),
                occurred_at: fixed_now,
            },
            kind: NarrativeEventKind::ChoiceSelected(ChoiceSelected {
                session_id,
                choice_label: "Go".to_owned(),
                from_scene_id: "start".to_owned(),
                to_scene_id: "next".to_owned(),
            }),
        };

        // Act
        session.apply(&event);

        // Assert
        assert!(session.active_choice_options.is_empty());
        assert_eq!(session.version, 1);
    }

    #[test]
    fn test_select_choice_produces_two_events() {
        // Arrange
        let session_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut session = NarrativeSession::new(session_id);
        let mut rng = MockRng;

        // Enter an initial scene first
        let scene_a = sample_scene_data(
            "start",
            vec![("Go north", "forest"), ("Go south", "village")],
        );
        session
            .enter_scene(&scene_a, Uuid::new_v4(), &clock, &mut rng)
            .unwrap();
        // Apply the event so aggregate state is updated
        let events: Vec<NarrativeEvent> = session.uncommitted_events().to_vec();
        for e in &events {
            session.apply(e);
        }
        session.clear_uncommitted_events();

        let target_scene = sample_scene_data("forest", vec![("Return", "start")]);

        // Act
        let result = session.select_choice(0, &target_scene, correlation_id, &clock, &mut rng);

        // Assert
        assert!(result.is_ok());
        let events = session.uncommitted_events();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_type(), "narrative.choice_selected");
        assert_eq!(events[1].event_type(), "narrative.scene_started");

        match &events[0].kind {
            NarrativeEventKind::ChoiceSelected(payload) => {
                assert_eq!(payload.choice_label, "Go north");
                assert_eq!(payload.from_scene_id, "start");
                assert_eq!(payload.to_scene_id, "forest");
            }
            other => panic!("expected ChoiceSelected, got {other:?}"),
        }

        match &events[1].kind {
            NarrativeEventKind::SceneStarted(payload) => {
                assert_eq!(payload.scene_id, "forest");
                assert_eq!(payload.choices.len(), 1);
            }
            other => panic!("expected SceneStarted, got {other:?}"),
        }
    }

    #[test]
    fn test_select_choice_rejects_archived_session() {
        // Arrange
        let session_id = Uuid::new_v4();
        let mut session = NarrativeSession::new(session_id);
        session.archived = true;
        let clock = FixedClock(Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap());
        let mut rng = MockRng;
        let target = sample_scene_data("next", vec![]);

        // Act
        let result = session.select_choice(0, &target, Uuid::new_v4(), &clock, &mut rng);

        // Assert
        match result.unwrap_err() {
            DomainError::Validation(msg) => assert_eq!(msg, "session is archived"),
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn test_select_choice_rejects_no_active_scene() {
        // Arrange
        let session_id = Uuid::new_v4();
        let mut session = NarrativeSession::new(session_id);
        let clock = FixedClock(Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap());
        let mut rng = MockRng;
        let target = sample_scene_data("next", vec![]);

        // Act
        let result = session.select_choice(0, &target, Uuid::new_v4(), &clock, &mut rng);

        // Assert
        match result.unwrap_err() {
            DomainError::Validation(msg) => assert_eq!(msg, "no active scene"),
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn test_select_choice_rejects_out_of_bounds_index() {
        // Arrange
        let session_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut session = NarrativeSession::new(session_id);
        let mut rng = MockRng;

        let scene = sample_scene_data("start", vec![("Go north", "forest")]);
        session
            .enter_scene(&scene, Uuid::new_v4(), &clock, &mut rng)
            .unwrap();
        let events: Vec<NarrativeEvent> = session.uncommitted_events().to_vec();
        for e in &events {
            session.apply(e);
        }
        session.clear_uncommitted_events();

        let target = sample_scene_data("village", vec![]);

        // Act — index 5 is out of bounds (only 1 choice available)
        let result = session.select_choice(5, &target, Uuid::new_v4(), &clock, &mut rng);

        // Assert
        match result.unwrap_err() {
            DomainError::Validation(msg) => {
                assert!(msg.contains("choice index 5 out of bounds"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn test_scene_history_tracks_multiple_transitions() {
        // Arrange
        let session_id = Uuid::new_v4();
        let fixed_now = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let clock = FixedClock(fixed_now);
        let mut session = NarrativeSession::new(session_id);
        let mut rng = MockRng;

        // Enter scene A
        let scene_a = sample_scene_data("A", vec![("Go to B", "B")]);
        session
            .enter_scene(&scene_a, Uuid::new_v4(), &clock, &mut rng)
            .unwrap();
        for e in session.uncommitted_events().to_vec() {
            session.apply(&e);
        }
        session.clear_uncommitted_events();

        // Select choice → B
        let scene_b = sample_scene_data("B", vec![("Go to C", "C")]);
        session
            .select_choice(0, &scene_b, Uuid::new_v4(), &clock, &mut rng)
            .unwrap();
        for e in session.uncommitted_events().to_vec() {
            session.apply(&e);
        }
        session.clear_uncommitted_events();

        // Select choice → C
        let scene_c = sample_scene_data("C", vec![]);
        session
            .select_choice(0, &scene_c, Uuid::new_v4(), &clock, &mut rng)
            .unwrap();
        for e in session.uncommitted_events().to_vec() {
            session.apply(&e);
        }
        session.clear_uncommitted_events();

        // Assert
        assert_eq!(session.current_scene_id, Some("C".to_owned()));
        assert_eq!(session.scene_history, vec!["A", "B", "C"]);
    }
}
