#!/usr/bin/env bash
set -euo pipefail

# Run all inline #[cfg(test)] unit tests across the Rust workspace.
# No database or external services required.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKEND_DIR="$SCRIPT_DIR/../backend"

cd "$BACKEND_DIR"

echo "=== Running Rust unit tests ==="
echo ""

cargo test --lib 2>&1
