/**
 * Base HTTP client for the Otherworlds backend API.
 *
 * This module lives under $lib/server/ so SvelteKit prevents
 * client-side imports. All fetch calls go through apiFetch<T>,
 * which handles JSON serialization, error parsing, and span-based
 * distributed tracing via OpenTelemetry.
 */

import { API_BASE_URL } from '$env/static/private';
import type { ErrorResponse } from '$lib/types';
import { getTracer, generateCorrelationId, SpanStatusCode } from '$lib/server/telemetry';

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
 * Each outgoing call is wrapped in an OpenTelemetry span with
 * standard HTTP semantic attributes.
 *
 * On non-OK responses the body is parsed as ErrorResponse and thrown
 * as an ApiClientError.
 */
async function apiFetch<T>(path: string, options: ApiFetchOptions = {}): Promise<T> {
  const { method = 'GET', body } = options;
  const correlationId = options.correlationId ?? generateCorrelationId();

  const url = `${API_BASE_URL}${path}`;
  const tracer = getTracer();

  return tracer.startActiveSpan(`api-client ${method} ${path}`, async (span) => {
    span.setAttribute('http.method', method);
    span.setAttribute('http.url', url);
    span.setAttribute('correlation.id', correlationId);

    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      'Accept': 'application/json',
      'X-Correlation-ID': correlationId,
    };

    const init: RequestInit = {
      method,
      headers,
    };

    if (body !== undefined) {
      init.body = JSON.stringify(body);
    }

    try {
      const response = await fetch(url, init);

      span.setAttribute('http.status_code', response.status);

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
        const err = new ApiClientError(response.status, errorResponse);
        span.setStatus({ code: SpanStatusCode.ERROR, message: err.message });
        span.recordException(err);
        span.end();
        throw err;
      }

      span.end();
      return (await response.json()) as T;
    } catch (error) {
      if (!(error instanceof ApiClientError)) {
        span.setStatus({ code: SpanStatusCode.ERROR });
        if (error instanceof Error) {
          span.recordException(error);
        }
        span.end();
      }
      throw error;
    }
  });
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
