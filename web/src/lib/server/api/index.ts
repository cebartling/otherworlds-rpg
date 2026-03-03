/**
 * Barrel file re-exporting all server-side API client modules.
 *
 * Usage:
 *   import { ApiClientError } from '$lib/server/api';
 *   import * as narrativeApi from '$lib/server/api/narrative';
 *
 * Or import specific modules via their namespace:
 *   import * as narrative from '$lib/server/api/narrative';
 *   import * as character from '$lib/server/api/character';
 */

export { ApiClientError, apiDelete, apiGet, apiPost } from './client';
export { handleLoadError, mapApiError } from './errors';
export * as narrative from './narrative';
export * as character from './character';
export * as rules from './rules';
export * as worldState from './world-state';
export * as inventory from './inventory';
export * as session from './session';
export * as content from './content';
