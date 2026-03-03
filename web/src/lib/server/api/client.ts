/**
 * Base HTTP client for the Otherworlds backend API.
 *
 * This module lives under $lib/server/ so SvelteKit prevents
 * client-side imports. All fetch calls go through apiFetch<T>,
 * which handles JSON serialization, error parsing, and dev logging.
 */

import { API_BASE_URL } from '$env/static/private';
import type { ErrorResponse } from '$lib/types';

/**
 * Typed error thrown when the API returns a non-OK status code.
 * Carries the parsed ErrorResponse body and the HTTP status.
 */
export class ApiClientError extends Error {
  public readonly status: number;
  public readonly errorResponse: ErrorResponse;

  constructor(status: number, errorResponse: ErrorResponse) {
    super(`API error ${status}: ${errorResponse.message}`);
    this.name = 'ApiClientError';
    this.status = status;
    this.errorResponse = errorResponse;
  }
}

/**
 * Options for apiFetch beyond the standard RequestInit fields.
 */
interface ApiFetchOptions {
  method?: string;
  body?: unknown;
  correlationId?: string;
}

/**
 * Core fetch wrapper that prepends API_BASE_URL, sets JSON headers,
 * forwards an optional correlation ID, and parses the response.
 *
 * On non-OK responses the body is parsed as ErrorResponse and thrown
 * as an ApiClientError.
 */
async function apiFetch<T>(path: string, options: ApiFetchOptions = {}): Promise<T> {
  const { method = 'GET', body, correlationId } = options;

  const url = `${API_BASE_URL}${path}`;

  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    'Accept': 'application/json',
  };

  if (correlationId) {
    headers['X-Correlation-ID'] = correlationId;
  }

  const init: RequestInit = {
    method,
    headers,
  };

  if (body !== undefined) {
    init.body = JSON.stringify(body);
  }

  if (import.meta.env.DEV) {
    console.log(`[api-client] ${method} ${url}`);
  }

  const response = await fetch(url, init);

  if (import.meta.env.DEV) {
    console.log(`[api-client] ${method} ${url} -> ${response.status}`);
  }

  if (!response.ok) {
    let errorResponse: ErrorResponse;
    try {
      errorResponse = (await response.json()) as ErrorResponse;
    } catch {
      errorResponse = {
        error: 'unknown_error',
        message: `HTTP ${response.status}: ${response.statusText}`,
      };
    }
    throw new ApiClientError(response.status, errorResponse);
  }

  return (await response.json()) as T;
}

/**
 * Perform a GET request against the API.
 */
export async function apiGet<T>(path: string, correlationId?: string): Promise<T> {
  return apiFetch<T>(path, { method: 'GET', correlationId });
}

/**
 * Perform a POST request against the API with a JSON body.
 */
export async function apiPost<T>(path: string, body: unknown, correlationId?: string): Promise<T> {
  return apiFetch<T>(path, { method: 'POST', body, correlationId });
}

/**
 * Perform a DELETE request against the API.
 */
export async function apiDelete<T>(path: string, correlationId?: string): Promise<T> {
  return apiFetch<T>(path, { method: 'DELETE', correlationId });
}
