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

The API server uses [tracing](https://docs.rs/tracing) for structured logging (JSON to stdout by default). To export traces via OpenTelemetry OTLP, set the collector endpoint:

```sh
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 cargo run -p otherworlds-api
```

When `OTEL_EXPORTER_OTLP_ENDPOINT` is set, an OTLP gRPC span exporter is added alongside the stdout layer. The service is registered as `otherworlds-api`. Any OpenTelemetry-compatible collector (Jaeger, Grafana Tempo, etc.) can receive the traces.

## Documentation

- [Product Manifesto](documentation/PRODUCT_MANIFESTO.md) — vision, principles, and design philosophy
- [Technical Manifesto](documentation/TECHNICAL_MANIFESTO.md) — architectural commitments and constraints

## License

[MIT](LICENSE)
