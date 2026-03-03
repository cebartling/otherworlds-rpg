/**
 * Types for the Character bounded context.
 *
 * Corresponds to Rust structs in:
 * - otherworlds-api/src/routes/character.rs (requests)
 * - otherworlds-character/src/application/query_handlers.rs (views)
 */

import type { UUID } from './common';

// ---------------------------------------------------------------------------
// Command / request types
// ---------------------------------------------------------------------------

/** Request body for POST /api/v1/characters/create. */
export interface CreateCharacterRequest {
  name: string;
}

/** Request body for POST /api/v1/characters/modify-attribute. */
export interface ModifyAttributeRequest {
  character_id: UUID;
  attribute: string;
  new_value: number;
}

/** Request body for POST /api/v1/characters/award-experience. */
export interface AwardExperienceRequest {
  character_id: UUID;
  amount: number;
}

// ---------------------------------------------------------------------------
// Query / view types
// ---------------------------------------------------------------------------

/** Full read-only view of a character (GET /api/v1/characters/:id). */
export interface CharacterView {
  character_id: UUID;
  name: string | null;
  attributes: Record<string, number>;
  experience: number;
  version: number;
}

/** Summary view for listing characters (GET /api/v1/characters). */
export interface CharacterSummary {
  character_id: UUID;
  name: string | null;
  experience: number;
  version: number;
}
