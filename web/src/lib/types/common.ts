/**
 * Common types shared across all bounded contexts.
 *
 * These types correspond to shared Rust structs in the
 * otherworlds-api crate (error responses, command responses, health).
 */

/** UUID string alias (v4/v7 format). */
export type UUID = string;

/** Standard error response body from the API. */
export interface ErrorResponse {
  error: string;
  message: string;
}

/**
 * Command response for contexts that return only event IDs.
 *
 * Used by: narrative, character, rules, world-state.
 */
export interface CommandResponse {
  event_ids: UUID[];
}

/**
 * Command response for contexts that also return the aggregate ID.
 *
 * Used by: session, inventory, content.
 */
export interface CommandResponseWithAggregate {
  aggregate_id: UUID;
  event_ids: UUID[];
}

/** Health check response from GET /health. */
export interface HealthResponse {
  status: string;
  version: string;
}
