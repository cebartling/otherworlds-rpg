# ADR-0023: Cross-Platform Observability via OpenTelemetry

## Status

Accepted

## Context

The Otherworlds RPG engine spans three platforms: a Rust/Axum backend, a SvelteKit web client, and a Swift/SwiftUI iOS client. Diagnosing issues in a multi-context, event-sourced system requires understanding the full request lifecycle — from user interaction through API orchestration to event persistence.

Without a unified observability strategy:

1. **Traces are platform-siloed** — backend logs don't connect to web or iOS request context.
2. **Performance bottlenecks are invisible** — no way to measure latency across the play loop's five phases.
3. **Development feedback is slow** — developers must manually correlate log output across services.

## Decision

All three platforms implement OpenTelemetry (OTel) tracing with a shared correlation ID (ADR-0022). Initialization is conditional — the system runs correctly without an OTel collector.

### Backend (Rust/Axum)

- Uses `tracing` crate with `tracing-subscriber` for structured JSON logging.
- When `OTEL_EXPORTER_OTLP_ENDPOINT` is set, an OpenTelemetry layer is stacked on top of the subscriber, exporting traces via gRPC (port 4317).
- When the environment variable is absent, only JSON logging is active — no OTel overhead.
- Every route handler is annotated with `#[instrument]` for automatic span creation.
- `TraceLayer` from `tower-http` instruments all HTTP requests.

### Web Client (SvelteKit)

- `NodeTracerProvider` with `BatchSpanProcessor` exports traces via HTTP (port 4318).
- Initialization in `telemetry.ts` is guarded by the presence of `OTEL_EXPORTER_OTLP_ENDPOINT`.
- `hooks.server.ts` creates a root span for every incoming request with HTTP semantic attributes.
- The server-side API client wraps each outbound call in a child span with method, path, status code, and correlation ID.

### iOS Client (SwiftUI)

- Uses Apple's `os.Logger` framework with three categorized subsystems: `api`, `ui`, and `general`.
- `HTTPClient` uses `OSSignposter` for Instruments profiling of network requests.
- Logs include correlation IDs, HTTP method, path, status code, and duration.
- No OTel SDK dependency — iOS observability stays native to Apple's tooling.

### Local Infrastructure

- Docker Compose includes a full observability stack (see ADR-0025): OTel Collector, Tempo (traces), Loki (logs), Prometheus (metrics), and Grafana (dashboards).
- The OTel Collector receives from both backend (gRPC) and web (HTTP) exporters.

## Consequences

### Easier

- End-to-end request tracing from web/iOS client through API orchestration to event persistence.
- Performance profiling of individual play loop phases via span timing.
- Local development includes full observability out of the box via Docker Compose.
- Conditional initialization means CI and lightweight development setups are not affected.

### More Difficult

- Three different tracing implementations must be kept conceptually aligned (same span naming conventions, same correlation ID propagation).
- The iOS client uses native Apple tooling rather than OTel, so cross-platform trace stitching requires correlation ID matching rather than W3C trace context propagation.
- Docker Compose stack is heavier with five additional containers for observability.

### Unchanged

- Application logic is unaffected by observability — tracing is purely additive.
- The determinism contract is maintained — tracing does not influence state transitions.
- Bounded context isolation is preserved — tracing spans are created at the API layer.
