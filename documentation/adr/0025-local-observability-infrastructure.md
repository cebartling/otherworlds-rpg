# ADR-0025: Local Observability Infrastructure via Docker Compose

## Status

Accepted

## Context

ADR-0023 establishes that all platforms emit OpenTelemetry traces and structured logs. These signals need a destination during local development. Without a local collector and visualization layer, developers must read raw log output or deploy to a staging environment to see traces.

The project already uses Docker Compose for PostgreSQL and the API server. Adding observability infrastructure to the same Compose file makes it available with a single `docker compose up`.

## Decision

The Docker Compose stack includes five observability containers, configured via files in `infra/telemetry/`:

### Components

1. **OpenTelemetry Collector** (`otel-collector`) — Receives traces from the backend (gRPC on port 4317) and web client (HTTP on port 4318). Exports to Tempo for storage.

2. **Grafana Tempo** (`tempo`) — Distributed tracing backend. Stores traces received from the OTel Collector. Queryable via Grafana's Tempo datasource.

3. **Grafana Loki** (`loki`) — Log aggregation. Receives logs and makes them searchable alongside traces in Grafana.

4. **Prometheus** (`prometheus`) — Metrics scraping. Configured to scrape the OTel Collector's metrics endpoint.

5. **Grafana** (`grafana`) — Unified dashboard. Pre-provisioned datasources for Tempo, Loki, and Prometheus. Anonymous access enabled for frictionless local use (port 3001).

### Configuration

- All telemetry configs live in `infra/telemetry/` (OTel collector config, Tempo config, Loki config, Prometheus scrape config, Grafana datasource provisioning).
- Container names follow the `otherworlds-rpg-` prefix convention.
- The observability stack starts alongside the application stack — no separate command needed.

### Conditional Backend Integration

- The backend only exports traces when `OTEL_EXPORTER_OTLP_ENDPOINT` is set in its environment. Docker Compose sets this automatically; running the binary standalone skips OTel export.

## Consequences

### Easier

- `docker compose up` gives developers full observability: traces, logs, and metrics in Grafana.
- Correlation IDs (ADR-0022) are visible in Tempo traces, enabling end-to-end request debugging.
- No external accounts or services needed — everything runs locally.
- Pre-provisioned Grafana datasources mean zero manual setup after `docker compose up`.

### More Difficult

- Five additional containers increase resource usage (memory, CPU, disk) during local development.
- Developers who don't need observability still download and start these containers unless they use `docker compose up postgres api` selectively.
- Configuration files in `infra/telemetry/` must be maintained as the observability stack evolves.

### Unchanged

- The application functions correctly without the observability stack — traces simply have no destination.
- CI pipelines are unaffected — they run tests without Docker Compose.
- Production deployment is independent — this stack is for local development only.
