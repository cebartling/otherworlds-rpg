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

## Documentation

- [Product Manifesto](documentation/PRODUCT_MANIFESTO.md) — vision, principles, and design philosophy
- [Technical Manifesto](documentation/TECHNICAL_MANIFESTO.md) — architectural commitments and constraints

## License

[MIT](LICENSE)
