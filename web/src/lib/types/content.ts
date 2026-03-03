/**
 * Types for the Content bounded context.
 *
 * Corresponds to Rust structs in:
 * - otherworlds-api/src/routes/content.rs (requests)
 * - otherworlds-content/src/application/query_handlers.rs (views)
 */

import type { UUID } from './common';

// ---------------------------------------------------------------------------
// Command / request types
// ---------------------------------------------------------------------------

/** Request body for POST /api/v1/content/ingest-campaign. */
export interface IngestCampaignRequest {
  source: string;
}

/** Request body for POST /api/v1/content/validate-campaign. */
export interface ValidateCampaignRequest {
  campaign_id: UUID;
}

/** Request body for POST /api/v1/content/compile-campaign. */
export interface CompileCampaignRequest {
  campaign_id: UUID;
}

// ---------------------------------------------------------------------------
// Query / view types
// ---------------------------------------------------------------------------

/** Full read-only view of a campaign (GET /api/v1/content/:id). */
export interface CampaignView {
  campaign_id: UUID;
  ingested: boolean;
  validated: boolean;
  compiled: boolean;
  version_hash: string | null;
  version: number;
}

/** Summary view for listing campaigns (GET /api/v1/content). */
export interface CampaignSummary {
  campaign_id: UUID;
  ingested: boolean;
  validated: boolean;
  compiled: boolean;
  version: number;
}
