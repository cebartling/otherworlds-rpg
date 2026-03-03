/**
 * Types for the Rules Engine bounded context.
 *
 * Corresponds to Rust structs in:
 * - otherworlds-api/src/routes/rules.rs (requests)
 * - otherworlds-rules/src/application/query_handlers.rs (views)
 */

import type { UUID } from './common';

// ---------------------------------------------------------------------------
// Command / request types
// ---------------------------------------------------------------------------

/** Request body for POST /api/v1/rules/declare-intent. */
export interface DeclareIntentRequest {
  resolution_id: UUID;
  intent_id: UUID;
  action_type: string;
  skill: string | null;
  target_id: UUID | null;
  difficulty_class: number;
  modifier: number;
}

/** Request body for POST /api/v1/rules/resolve-check. */
export interface ResolveCheckRequest {
  resolution_id: UUID;
}

/** A single effect specification within a ProduceEffectsRequest. */
export interface EffectSpecRequest {
  effect_type: string;
  target_id: UUID | null;
  payload: unknown;
}

/** Request body for POST /api/v1/rules/produce-effects. */
export interface ProduceEffectsRequest {
  resolution_id: UUID;
  effects: EffectSpecRequest[];
}

// ---------------------------------------------------------------------------
// Query / view types
// ---------------------------------------------------------------------------

/** View of a declared intent within a resolution. */
export interface IntentView {
  intent_id: UUID;
  action_type: string;
  skill: string | null;
  target_id: UUID | null;
  difficulty_class: number;
  modifier: number;
}

/** View of a resolved check within a resolution. */
export interface CheckResultView {
  check_id: UUID;
  natural_roll: number;
  modifier: number;
  total: number;
  difficulty_class: number;
  outcome: string;
}

/** View of a produced effect within a resolution. */
export interface EffectView {
  effect_type: string;
  target_id: UUID | null;
  payload: unknown;
}

/** Full read-only view of a resolution (GET /api/v1/rules/:id). */
export interface ResolutionView {
  resolution_id: UUID;
  phase: string;
  intent: IntentView | null;
  check_result: CheckResultView | null;
  effects: EffectView[];
  version: number;
}

/** Summary view for listing resolutions (GET /api/v1/rules). */
export interface ResolutionSummary {
  resolution_id: UUID;
  phase: string;
  version: number;
}
