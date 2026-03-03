/**
 * Types for the Session bounded context.
 *
 * Corresponds to Rust structs in:
 * - otherworlds-api/src/routes/session.rs (requests)
 * - otherworlds-session/src/application/query_handlers.rs (views)
 */

import type { UUID } from './common';

// ---------------------------------------------------------------------------
// Command / request types
// ---------------------------------------------------------------------------

/** Request body for POST /api/v1/sessions/start-campaign-run. */
export interface StartCampaignRunRequest {
  campaign_id: UUID;
}

/** Request body for POST /api/v1/sessions/create-checkpoint. */
export interface CreateCheckpointRequest {
  run_id: UUID;
}

/** Request body for POST /api/v1/sessions/branch-timeline. */
export interface BranchTimelineRequest {
  source_run_id: UUID;
  from_checkpoint_id: UUID;
}

// ---------------------------------------------------------------------------
// Query / view types
// ---------------------------------------------------------------------------

/** Full read-only view of a campaign run (GET /api/v1/sessions/:id). */
export interface CampaignRunView {
  run_id: UUID;
  campaign_id: UUID | null;
  checkpoint_ids: UUID[];
  version: number;
}

/** Summary view for listing campaign runs (GET /api/v1/sessions). */
export interface CampaignRunSummary {
  run_id: UUID;
  campaign_id: UUID | null;
  checkpoint_count: number;
  version: number;
}
