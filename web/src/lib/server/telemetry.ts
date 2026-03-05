/**
 * OpenTelemetry initialization for the Otherworlds SvelteKit server.
 *
 * Configures a NodeTracerProvider with OTLP HTTP export when the
 * OTEL_EXPORTER_OTLP_ENDPOINT environment variable is set. When the
 * variable is absent, all tracing calls become no-ops.
 */

import { trace, SpanStatusCode } from '@opentelemetry/api';
import type { Tracer, Span } from '@opentelemetry/api';
import { NodeTracerProvider } from '@opentelemetry/sdk-trace-node';
import { BatchSpanProcessor } from '@opentelemetry/sdk-trace-node';
import { OTLPTraceExporter } from '@opentelemetry/exporter-trace-otlp-http';
import { resourceFromAttributes } from '@opentelemetry/resources';
import { ATTR_SERVICE_NAME, ATTR_SERVICE_VERSION } from '@opentelemetry/semantic-conventions';

const SERVICE_NAME = 'otherworlds-web';

let initialized = false;

function initTelemetry(): void {
  if (initialized) return;
  initialized = true;

  const endpoint = process.env.OTEL_EXPORTER_OTLP_ENDPOINT;
  if (!endpoint) return;

  const resource = resourceFromAttributes({
    [ATTR_SERVICE_NAME]: SERVICE_NAME,
    [ATTR_SERVICE_VERSION]: '0.0.1',
  });

  const exporter = new OTLPTraceExporter({
    url: `${endpoint}/v1/traces`,
  });

  const provider = new NodeTracerProvider({
    resource,
    spanProcessors: [new BatchSpanProcessor(exporter)],
  });

  provider.register();
}

// Initialize on module load (server-side only).
initTelemetry();

/**
 * Returns a tracer instance. If OTel is not configured the global
 * trace API returns a no-op tracer automatically.
 */
export function getTracer(): Tracer {
  return trace.getTracer(SERVICE_NAME);
}

/**
 * Generate a UUID v4 string suitable for use as a correlation ID.
 */
export function generateCorrelationId(): string {
  return crypto.randomUUID();
}

// Re-export useful OTel types for consumers.
export { SpanStatusCode };
export type { Span };
