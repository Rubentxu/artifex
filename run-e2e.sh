#!/bin/bash
# E2E Test Runner for Artifex
# Usage: ./run-e2e.sh [--skip-build]
#
# Prerequisites:
#   - cargo build (debug mode)
#   - tauri-driver installed (cargo install tauri-driver --locked)
#   - webkit2gtk-driver installed (Linux: sudo apt install webkit2gtk-driver)
#   - npm install in e2e-tests/

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
E2E_DIR="$SCRIPT_DIR/e2e-tests"

# Check prerequisites
if ! command -v tauri-driver &>/dev/null; then
  echo "❌ tauri-driver not found. Install with: cargo install tauri-driver --locked"
  exit 1
fi

if [ ! -d "$E2E_DIR/node_modules" ]; then
  echo "📦 Installing E2E dependencies..."
  cd "$E2E_DIR" && npm install
fi

# Build if not skipped
if [ "$1" != "--skip-build" ]; then
  echo "🔧 Building Artifex (debug)..."
  cd "$SCRIPT_DIR" && cargo build
fi

# Check binary exists
BINARY="$SCRIPT_DIR/target/debug/src-tauri"
if [ ! -f "$BINARY" ]; then
  echo "❌ Binary not found at $BINARY"
  exit 1
fi

echo "🧪 Running E2E tests..."
cd "$E2E_DIR" && npx wdio run wdio.conf.js

echo "✅ E2E tests complete!"
