/**
 * Server-side error mapping for SvelteKit load functions and actions.
 *
 * Translates ApiClientError (from the backend HTTP client) into
 * SvelteKit's error() responses with appropriate HTTP status codes.
 * This module lives under $lib/server/ because it imports from
 * the API client which uses private environment variables.
 */

import { error } from '@sveltejs/kit';
import { ApiClientError } from './client';

/**
 * Known backend error codes from the Rust API's DomainError variants.
 */
const ERROR_CODE_STATUS_MAP: Record<string, number> = {
  aggregate_not_found: 404,
  concurrency_conflict: 409,
  validation_error: 400,
  infrastructure_error: 500,
};

/**
 * Maps an API error to a SvelteKit error response.
 *
 * When the error is an ApiClientError, it inspects the error code
 * from the response body first, then falls back to the HTTP status.
 * Non-ApiClientError exceptions are treated as 500 Internal Server Error.
 *
 * @param err - The caught error from an API call
 * @throws Always throws a SvelteKit error() — never returns
 */
export function mapApiError(err: unknown): never {
  if (err instanceof ApiClientError) {
    const code = err.errorResponse.error;
    const message = err.errorResponse.message;

    // Prefer the error code mapping over raw HTTP status
    const mappedStatus = ERROR_CODE_STATUS_MAP[code];
    if (mappedStatus !== undefined) {
      error(mappedStatus, { message });
    }

    // Fall back to HTTP status for unmapped codes (e.g. 422 from Axum)
    if (err.status === 404) {
      error(404, { message });
    }
    if (err.status === 409) {
      error(409, { message });
    }
    if (err.status === 400 || err.status === 422) {
      error(400, { message });
    }

    // Everything else is an internal error
    error(500, { message });
  }

  // Non-API errors (network failures, unexpected exceptions, etc.)
  const message = err instanceof Error ? err.message : 'An unexpected error occurred';
  error(500, { message });
}

/**
 * Convenience wrapper for use in SvelteKit load functions.
 *
 * Usage:
 * ```ts
 * export const load: PageServerLoad = async () => {
 *   try {
 *     const data = await apiGet('/some/path');
 *     return { data };
 *   } catch (err) {
 *     handleLoadError(err);
 *   }
 * };
 * ```
 *
 * @param err - The caught error from an API call
 * @throws Always throws a SvelteKit error() — never returns
 */
export function handleLoadError(err: unknown): never {
  mapApiError(err);
}
