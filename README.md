# Otherworlds RPG

A deterministic, event-driven narrative engine for tabletop RPG experiences.

Every action is a command. Every resolution is explicit. Every outcome becomes a recorded fact.

## Monorepo Structure

```
otherworlds-rpg/
├── backend/          Rust (Axum/Tokio) modular monolith
├── web/              SvelteKit web client
├── ios/              Swift/SwiftUI iOS client
└── documentation/    Product and technical manifestos
```

## Prerequisites

- [Rust](https://rustup.rs/) (stable, 1.85+)
- [Node.js](https://nodejs.org/) (22+)
- [Docker](https://www.docker.com/) & Docker Compose

## Quick Start

### Backend

```sh
cd backend
cargo build
cargo run -p otherworlds-api
```

### Database (PostgreSQL)

```sh
docker compose up postgres -d
```

### Full Stack (Docker)

```sh
docker compose up
```

The API server will be available at `http://localhost:3000/health`.

### Web Client

```sh
cd web
npm install
npm run dev
```

### iOS Client

Open `ios/OtherworldsApp` in Xcode or build with Swift Package Manager:

```sh
cd ios/OtherworldsApp
swift build
```

## Observability

The full stack includes a Grafana LGTM telemetry pipeline. Running `docker compose up` starts an OpenTelemetry Collector, Tempo (traces), Loki (logs), Prometheus (metrics), and Grafana (visualization).

| Service | URL | Purpose |
|---------|-----|---------|
| Grafana | http://localhost:3001 | Dashboards and trace explorer |
| Prometheus | http://localhost:9090 | Metrics query UI |
| OTel Collector | localhost:4317 (gRPC), localhost:4318 (HTTP) | OTLP receiver |

### Viewing traces

1. `docker compose up`
2. Hit `http://localhost:3000/health` to generate a trace.
3. Open Grafana at `http://localhost:3001`.
4. Navigate to **Explore** and select the **Tempo** data source.
5. Search for traces from `otherworlds-api` or `otherworlds-web`.

Grafana logs in with admin/admin by default (anonymous access is also enabled).

### Local development without Docker

To export traces from a local `cargo run` to the collector running in Docker:

```sh
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 cargo run -p otherworlds-api
```

When `OTEL_EXPORTER_OTLP_ENDPOINT` is set, an OTLP gRPC span exporter is added alongside the stdout layer. The service is registered as `otherworlds-api`.

## Documentation

- [Product Manifesto](documentation/PRODUCT_MANIFESTO.md) — vision, principles, and design philosophy
- [Technical Manifesto](documentation/TECHNICAL_MANIFESTO.md) — architectural commitments and constraints

## License

[MIT](LICENSE)
