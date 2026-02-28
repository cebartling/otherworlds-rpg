//! Domain events for the Session & Progress context.

use otherworlds_core::event::{DomainEvent, EventMetadata};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Emitted when a campaign run is started.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignRunStarted {
    /// The campaign run identifier.
    pub run_id: Uuid,
    /// The campaign identifier.
    pub campaign_id: Uuid,
}

/// Emitted when a checkpoint is created.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointCreated {
    /// The campaign run identifier.
    pub run_id: Uuid,
    /// The checkpoint identifier.
    pub checkpoint_id: Uuid,
}

/// Emitted when a timeline is branched.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineBranched {
    /// The original campaign run identifier.
    pub source_run_id: Uuid,
    /// The new branch's campaign run identifier.
    pub branch_run_id: Uuid,
    /// The checkpoint to branch from.
    pub from_checkpoint_id: Uuid,
}

/// Event payload variants for the Session & Progress context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionEventKind {
    /// A campaign run has started.
    CampaignRunStarted(CampaignRunStarted),
    /// A checkpoint has been created.
    CheckpointCreated(CheckpointCreated),
    /// A timeline has been branched.
    TimelineBranched(TimelineBranched),
}

/// Domain event envelope for the Session & Progress context.
#[derive(Debug, Clone)]
pub struct SessionEvent {
    /// Event metadata.
    pub metadata: EventMetadata,
    /// Event-specific payload.
    pub kind: SessionEventKind,
}

impl DomainEvent for SessionEvent {
    fn event_type(&self) -> &'static str {
        match &self.kind {
            SessionEventKind::CampaignRunStarted(_) => "session.campaign_run_started",
            SessionEventKind::CheckpointCreated(_) => "session.checkpoint_created",
            SessionEventKind::TimelineBranched(_) => "session.timeline_branched",
        }
    }

    fn to_payload(&self) -> serde_json::Value {
        // Serialization of derived Serialize types to Value is infallible.
        serde_json::to_value(&self.kind).expect("SessionEventKind serialization is infallible")
    }

    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
}
