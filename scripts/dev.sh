#!/usr/bin/env bash
set -euo pipefail

# Full-stack dev launcher: Docker services + env validation + SvelteKit dev server.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$SCRIPT_DIR/.."
WEB_DIR="$REPO_ROOT/web"

# --- Step 1: Start Docker services ---

echo "=== Starting Docker services ==="

cd "$REPO_ROOT"
docker compose up -d

# --- Step 2: Wait for PostgreSQL ---

echo ""
echo "=== Waiting for PostgreSQL ==="

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

# --- Step 3: Wait for API health ---

echo ""
echo "=== Waiting for API server ==="

API_URL="http://localhost:3000/health"
API_MAX_RETRIES=15
API_RETRY_DELAY=2

for i in $(seq 1 "$API_MAX_RETRIES"); do
    if curl -sf "$API_URL" >/dev/null 2>&1; then
        echo "API server is healthy."
        break
    fi
    if [ "$i" -eq "$API_MAX_RETRIES" ]; then
        echo "ERROR: API server failed to respond at $API_URL after $API_MAX_RETRIES attempts." >&2
        echo "Check logs with: docker compose logs api" >&2
        exit 1
    fi
    echo "Waiting for API... (attempt $i/$API_MAX_RETRIES)"
    sleep "$API_RETRY_DELAY"
done

# --- Step 4: Check web/.env ---

echo ""
echo "=== Checking web environment ==="

if [ ! -f "$WEB_DIR/.env" ]; then
    if [ -f "$WEB_DIR/.env.example" ]; then
        cp "$WEB_DIR/.env.example" "$WEB_DIR/.env"
        echo "Created web/.env from web/.env.example"
    else
        echo "ERROR: web/.env.example not found. Create web/.env manually with:" >&2
        echo "  API_BASE_URL=http://localhost:3000" >&2
        exit 1
    fi
fi

# --- Step 5: Validate required env vars ---

# Source .env (handle lines with spaces in values)
set -a
# shellcheck source=/dev/null
source "$WEB_DIR/.env"
set +a

MISSING_VARS=()
if [ -z "${API_BASE_URL:-}" ]; then
    MISSING_VARS+=("API_BASE_URL")
fi

if [ ${#MISSING_VARS[@]} -gt 0 ]; then
    echo "ERROR: Missing required env vars in web/.env: ${MISSING_VARS[*]}" >&2
    exit 1
fi

echo "Environment OK (API_BASE_URL=$API_BASE_URL)"

# --- Step 6: Install web dependencies if needed ---

if [ ! -d "$WEB_DIR/node_modules" ]; then
    echo ""
    echo "=== Installing web dependencies ==="
    cd "$WEB_DIR"
    npm install
fi

# --- Step 7: Print summary and start SvelteKit ---

echo ""
echo "========================================="
echo "  Otherworlds RPG — Dev Stack Running"
echo "========================================="
echo "  API:  http://localhost:3000"
echo "  Web:  http://localhost:5173"
echo "========================================="
echo ""
echo "Starting SvelteKit dev server... (Ctrl+C to stop)"
echo ""

cd "$WEB_DIR"
exec npm run dev
