# Artifex — Task runner
# https://github.com/casey/just
#
# Quick start:
#   just              # show all recipes
#   just dev          # start Tauri dev (hot-reload frontend + backend)
#   just build        # full production build
#   just test         # run all tests (Rust + frontend + E2E)
#   just check        # fast lint check

# ── Config ────────────────────────────────────────────────────
set dotenv-load
set positional-arguments

# Project directories
src_dir   := justfile_directory() / "src"
tauri_dir := justfile_directory() / "src-tauri"
e2e_dir   := justfile_directory() / "e2e-tests"
target_dir := justfile_directory() / "target"

# Binary name
binary := target_dir / "debug" / "src-tauri"

# ════════════════════════════════════════════════════════════════
#  Development
# ════════════════════════════════════════════════════════════════

# Start Tauri in dev mode (Vite dev server + Rust hot-reload)
dev:
    cargo tauri dev

# Start only the Vite frontend dev server (no Rust backend)
dev-frontend:
    cd {{ src_dir }} && npm run dev

# Start Vite + open browser preview (no Tauri window)
preview: build-frontend
    cd {{ src_dir }} && npm run preview

# ════════════════════════════════════════════════════════════════
#  Building
# ════════════════════════════════════════════════════════════════

# Full production build (Rust release + frontend)
build: build-frontend
    cargo tauri build

# Debug build (faster, no optimizations)
build-debug: build-frontend-dev
    cargo build

# Build only the Rust workspace (no frontend)
build-rust:
    cargo build --workspace

# Build frontend for production
build-frontend:
    cd {{ src_dir }} && npm run build

# Build frontend in dev mode (no minification)
build-frontend-dev:
    cd {{ src_dir }} && npx vite build --mode development

# Build frontend for E2E tests (dev mode + debug harness enabled)
build-frontend-e2e:
    cd {{ src_dir }} && ARTIFEX_E2E=1 npx vite build --mode development

# Debug build for E2E (frontend with harness + Rust debug binary)
build-debug-e2e: build-frontend-e2e
    cargo build

# ════════════════════════════════════════════════════════════════
#  Testing
# ════════════════════════════════════════════════════════════════

# Run all tests: Rust + Frontend unit + E2E
test: test-rust test-frontend test-e2e

# Run Rust workspace tests
test-rust:
    cargo test --workspace

# Run Rust tests with output
test-rust-verbose:
    cargo test --workspace -- --nocapture

# Run frontend unit tests (vitest)
test-frontend:
    cd {{ src_dir }} && npm run test

# Run E2E tests with tauri-driver (headless via xvfb)
test-e2e: build-debug-e2e
    cd {{ e2e_dir }} && xvfb-run npx wdio run wdio.conf.js

# Run E2E tests headed (with GUI, for debugging)
test-e2e-headed: build-debug-e2e
    cd {{ e2e_dir }} && npx wdio run wdio.conf.js --watch

# Run E2E tests with a specific spec file
test-e2e-spec spec: build-debug-e2e
    cd {{ e2e_dir }} && xvfb-run npx wdio run wdio.conf.js --spec "{{ spec }}"

# Run a single E2E spec by number (e.g., just test-e2e-one 01)
test-e2e-one number: build-debug-e2e
    cd {{ e2e_dir }} && xvfb-run npx wdio run wdio.conf.js --spec "specs/{{ number }}-*.js"

# Run E2E tests for AI mock features (specs 11-23)
test-e2e-mock: build-debug-e2e
    cd {{ e2e_dir }} && xvfb-run npx wdio run wdio.conf.js --spec "specs/1[1-9]-*.js" --spec "specs/2[0-3]-*.js"

# Run E2E tests for base features only (specs 01-10)
test-e2e-base: build-debug-e2e
    cd {{ e2e_dir }} && xvfb-run npx wdio run wdio.conf.js --spec "specs/0[1-9]-*.js" --spec "specs/10-*.js"

# ════════════════════════════════════════════════════════════════
#  Checking & Linting
# ════════════════════════════════════════════════════════════════

# Fast check: cargo check + frontend build (no tests)
check: check-rust check-frontend

# Cargo check (fast Rust compilation check)
check-rust:
    cargo check --workspace

# Cargo clippy (linter)
clippy:
    cargo clippy --workspace -- -D warnings

# Frontend type check + build
check-frontend: build-frontend

# Svelte type checking
check-svelte:
    cd {{ src_dir }} && npx svelte-check --tsconfig ./tsconfig.json

# ════════════════════════════════════════════════════════════════
#  Setup & Dependencies
# ════════════════════════════════════════════════════════════════

# Install all dependencies (Rust + frontend + E2E)
setup: setup-rust setup-frontend setup-e2e

# Install Rust toolchain and build dependencies
setup-rust:
    cargo build --workspace

# Install frontend dependencies
setup-frontend:
    cd {{ src_dir }} && npm install

# Install E2E test dependencies
setup-e2e:
    cd {{ e2e_dir }} && npm install

# Check if tauri-driver is installed
check-tauri-driver:
    @test -f ~/.cargo/bin/tauri-driver && echo "✓ tauri-driver installed" || (echo "✗ tauri-driver missing — install with: cargo install tauri-driver" && exit 1)

# Check if xvfb-run is available
check-xvfb:
    @which xvfb-run > /dev/null && echo "✓ xvfb-run available" || (echo "✗ xvfb-run missing — install with: sudo apt install xvfb" && exit 1)

# ════════════════════════════════════════════════════════════════
#  Cleanup
# ════════════════════════════════════════════════════════════════

# Clean all build artifacts
clean:
    cargo clean --workspace
    cd {{ src_dir }} && rm -rf build .svelte-kit node_modules/.vite
    cd {{ e2e_dir }} && rm -rf node_modules/.cache
    echo "✓ Cleaned"

# Clean Rust build only
clean-rust:
    cargo clean --workspace

# Clean frontend build only
clean-frontend:
    cd {{ src_dir }} && rm -rf build .svelte-kit

# ════════════════════════════════════════════════════════════════
#  Git & Release
# ════════════════════════════════════════════════════════════════

# Show current status summary
status:
    @echo "═══ Artifex Status ═══"
    @echo ""
    @echo "Rust:"
    @cargo check --workspace 2>&1 | tail -1
    @echo ""
    @echo "Tests:"
    @cargo test --workspace 2>&1 | grep "^test result" | tail -5
    @cd {{ src_dir }} && npm run test 2>&1 | grep -E "Tests|Test Files" | tail -2
    @echo ""
    @echo "Git:"
    @git log --oneline -5
    @echo ""
    @git status --short

# Quick pre-flight check before committing
preflight: check test-rust test-frontend
    @echo "✓ Preflight check passed — ready to commit"

# ════════════════════════════════════════════════════════════════
#  Info
# ════════════════════════════════════════════════════════════════

# Show project info
info:
    @echo "╔══════════════════════════════════════════════╗"
    @echo "║  Artifex — Game AI Studio                    ║"
    @echo "║  Rust + Tauri v2 + SvelteKit                 ║"
    @echo "╚══════════════════════════════════════════════╝"
    @echo ""
    @echo "Commands:"
    @echo "  just dev              Start dev mode (hot-reload)"
    @echo "  just build            Production build"
    @echo "  just test             All tests (Rust + Frontend + E2E)"
    @echo "  just test-e2e         E2E tests only"
    @echo "  just test-e2e-one 01  Run spec 01 only"
    @echo "  just check            Fast compilation check"
    @echo "  just status           Show project status"
    @echo "  just preflight        Pre-commit check"
    @echo ""
    @echo "E2E test groups:"
    @echo "  just test-e2e-base    Specs 01-10 (app structure)"
    @echo "  just test-e2e-mock    Specs 11-23 (AI mock features)"
    @echo ""

# Default: show info
default: info
