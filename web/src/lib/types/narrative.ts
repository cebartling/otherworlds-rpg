/**
 * Types for the Narrative Orchestration bounded context.
 *
 * Corresponds to Rust structs in:
 * - otherworlds-api/src/routes/narrative.rs (requests)
 * - otherworlds-narrative/src/application/query_handlers.rs (views)
 * - otherworlds-narrative/src/domain/value_objects.rs (value objects)
 */

import type { UUID } from './common';

// ---------------------------------------------------------------------------
// Value objects
// ---------------------------------------------------------------------------

/** A single choice option within a scene. */
export interface ChoiceOption {
  label: string;
  target_scene_id: string;
}

// ---------------------------------------------------------------------------
// Command / request types
// ---------------------------------------------------------------------------

/** Request body for POST /api/v1/narrative/advance-beat. */
export interface AdvanceBeatRequest {
  session_id: UUID;
}

/** Request body for POST /api/v1/narrative/present-choice. */
export interface PresentChoiceRequest {
  session_id: UUID;
}

/** Choice option within an EnterSceneRequest. */
export interface ChoiceOptionRequest {
  label: string;
  target_scene_id: string;
}

/** Request body for POST /api/v1/narrative/enter-scene. */
export interface EnterSceneRequest {
  session_id: UUID;
  scene_id: string;
  narrative_text: string;
  choices: ChoiceOptionRequest[];
  npc_refs?: string[];
}

/** Target scene data within a SelectChoiceRequest. */
export interface TargetSceneRequest {
  scene_id: string;
  narrative_text: string;
  choices: ChoiceOptionRequest[];
  npc_refs?: string[];
}

/** Request body for POST /api/v1/narrative/select-choice. */
export interface SelectChoiceRequest {
  session_id: UUID;
  choice_index: number;
  target_scene: TargetSceneRequest;
}

// ---------------------------------------------------------------------------
// Query / view types
// ---------------------------------------------------------------------------

/** Full read-only view of a narrative session (GET /api/v1/narrative/:id). */
export interface NarrativeSessionView {
  session_id: UUID;
  current_beat_id: UUID | null;
  choice_ids: UUID[];
  current_scene_id: string | null;
  scene_history: string[];
  active_choice_options: ChoiceOption[];
  version: number;
}

/** Summary view for listing narrative sessions (GET /api/v1/narrative). */
export interface NarrativeSessionSummary {
  session_id: UUID;
  current_beat_id: UUID | null;
  current_scene_id: string | null;
  version: number;
}
