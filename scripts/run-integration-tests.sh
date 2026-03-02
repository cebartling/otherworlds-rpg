#!/usr/bin/env bash
set -euo pipefail

# Run all integration tests (tests/ directories) across the Rust workspace.
# Requires PostgreSQL — starts it via docker compose if not already running.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$SCRIPT_DIR/.."
BACKEND_DIR="$REPO_ROOT/backend"

DATABASE_URL="${DATABASE_URL:-postgres://otherworlds:otherworlds@localhost:5432/otherworlds}"
export DATABASE_URL

# --- Step 1: Ensure PostgreSQL is running ---

echo "=== Ensuring PostgreSQL is running ==="

cd "$REPO_ROOT"
docker compose up postgres -d

MAX_RETRIES=5
RETRY_DELAY=2

for i in $(seq 1 "$MAX_RETRIES"); do
    if docker compose exec postgres pg_isready -U otherworlds >/dev/null 2>&1; then
        echo "PostgreSQL is ready."
        break
    fi
    if [ "$i" -eq "$MAX_RETRIES" ]; then
        echo "ERROR: PostgreSQL failed to become ready after $MAX_RETRIES attempts." >&2
        exit 1
    fi
    echo "Waiting for PostgreSQL... (attempt $i/$MAX_RETRIES)"
    sleep "$RETRY_DELAY"
done

# --- Step 2: Run integration tests ---

echo ""
echo "=== Running Rust integration tests ==="
echo ""

cd "$BACKEND_DIR"

cargo test --test '*' 2>&1
