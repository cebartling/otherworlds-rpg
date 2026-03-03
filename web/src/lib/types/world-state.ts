/**
 * Types for the World State bounded context.
 *
 * Corresponds to Rust structs in:
 * - otherworlds-api/src/routes/world_state.rs (requests)
 * - otherworlds-world-state/src/application/query_handlers.rs (views)
 */

import type { UUID } from './common';

// ---------------------------------------------------------------------------
// Command / request types
// ---------------------------------------------------------------------------

/** Request body for POST /api/v1/world-state/apply-effect. */
export interface ApplyEffectRequest {
  world_id: UUID;
  fact_key: string;
}

/** Request body for POST /api/v1/world-state/set-flag. */
export interface SetFlagRequest {
  world_id: UUID;
  flag_key: string;
  value: boolean;
}

/** Request body for POST /api/v1/world-state/update-disposition. */
export interface UpdateDispositionRequest {
  world_id: UUID;
  entity_id: UUID;
}

// ---------------------------------------------------------------------------
// Query / view types
// ---------------------------------------------------------------------------

/** Full read-only view of a world snapshot (GET /api/v1/world-state/:id). */
export interface WorldSnapshotView {
  world_id: UUID;
  facts: string[];
  flags: Record<string, boolean>;
  disposition_entity_ids: UUID[];
  version: number;
}

/** Summary view for listing world snapshots (GET /api/v1/world-state). */
export interface WorldSnapshotSummary {
  world_id: UUID;
  fact_count: number;
  flag_count: number;
  version: number;
}
