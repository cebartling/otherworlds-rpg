import { API_BASE_URL } from '$env/static/private';
import type { Handle } from '@sveltejs/kit';
import { getTracer, generateCorrelationId, SpanStatusCode } from '$lib/server/telemetry';

// Validate required environment variables on startup.
// This runs once when the server-side module is first loaded.
if (!API_BASE_URL) {
  throw new Error(
    'Missing required environment variable: API_BASE_URL. ' +
    'Set it in web/.env (e.g. API_BASE_URL=http://localhost:3000).'
  );
}

export const handle: Handle = async ({ event, resolve }) => {
  const tracer = getTracer();
  const correlationId = generateCorrelationId();
  event.locals.correlationId = correlationId;

  return tracer.startActiveSpan(
    `${event.request.method} ${event.url.pathname}`,
    async (span) => {
      const start = performance.now();

      span.setAttribute('http.method', event.request.method);
      span.setAttribute('http.url', event.url.toString());
      span.setAttribute('http.route', event.url.pathname);
      span.setAttribute('correlation.id', correlationId);

      try {
        const response = await resolve(event);

        span.setAttribute('http.status_code', response.status);
        span.setAttribute('http.duration_ms', Math.round(performance.now() - start));
        span.end();

        return response;
      } catch (error) {
        span.setAttribute('http.duration_ms', Math.round(performance.now() - start));
        span.setStatus({ code: SpanStatusCode.ERROR });
        if (error instanceof Error) {
          span.recordException(error);
        }
        span.end();
        throw error;
      }
    },
  );
};
