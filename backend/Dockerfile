# Stage 1: Build
FROM rust:1.86-slim AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y pkg-config && rm -rf /var/lib/apt/lists/*

# Copy workspace manifests first for layer caching.
COPY Cargo.toml Cargo.lock ./
COPY crates/otherworlds-core/Cargo.toml crates/otherworlds-core/Cargo.toml
COPY crates/otherworlds-event-store/Cargo.toml crates/otherworlds-event-store/Cargo.toml
COPY crates/otherworlds-narrative/Cargo.toml crates/otherworlds-narrative/Cargo.toml
COPY crates/otherworlds-rules/Cargo.toml crates/otherworlds-rules/Cargo.toml
COPY crates/otherworlds-world-state/Cargo.toml crates/otherworlds-world-state/Cargo.toml
COPY crates/otherworlds-character/Cargo.toml crates/otherworlds-character/Cargo.toml
COPY crates/otherworlds-inventory/Cargo.toml crates/otherworlds-inventory/Cargo.toml
COPY crates/otherworlds-session/Cargo.toml crates/otherworlds-session/Cargo.toml
COPY crates/otherworlds-content/Cargo.toml crates/otherworlds-content/Cargo.toml
COPY crates/otherworlds-api/Cargo.toml crates/otherworlds-api/Cargo.toml

# Create dummy source files to cache dependency compilation.
RUN mkdir -p crates/otherworlds-core/src && echo "//! placeholder" > crates/otherworlds-core/src/lib.rs \
    && mkdir -p crates/otherworlds-event-store/src && echo "//! placeholder" > crates/otherworlds-event-store/src/lib.rs \
    && mkdir -p crates/otherworlds-narrative/src && echo "//! placeholder" > crates/otherworlds-narrative/src/lib.rs \
    && mkdir -p crates/otherworlds-rules/src && echo "//! placeholder" > crates/otherworlds-rules/src/lib.rs \
    && mkdir -p crates/otherworlds-world-state/src && echo "//! placeholder" > crates/otherworlds-world-state/src/lib.rs \
    && mkdir -p crates/otherworlds-character/src && echo "//! placeholder" > crates/otherworlds-character/src/lib.rs \
    && mkdir -p crates/otherworlds-inventory/src && echo "//! placeholder" > crates/otherworlds-inventory/src/lib.rs \
    && mkdir -p crates/otherworlds-session/src && echo "//! placeholder" > crates/otherworlds-session/src/lib.rs \
    && mkdir -p crates/otherworlds-content/src && echo "//! placeholder" > crates/otherworlds-content/src/lib.rs \
    && mkdir -p crates/otherworlds-api/src && echo "fn main() {}" > crates/otherworlds-api/src/main.rs

RUN cargo build --release 2>/dev/null || true

# Copy actual source code.
COPY crates/ crates/

# Touch source files to invalidate cached compilation.
RUN find crates -name "*.rs" -exec touch {} +

RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/otherworlds-api /app/otherworlds-api

EXPOSE 3000

ENV RUST_LOG=info

CMD ["/app/otherworlds-api"]
