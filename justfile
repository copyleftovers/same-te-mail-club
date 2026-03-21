# samete development commands

default:
    @just --list

# Start dev server with hot reload
dev:
    cargo leptos watch

# Run unit tests
test:
    cargo test

# Run clippy on both SSR and hydrate targets
clippy:
    cargo clippy --features ssr --no-default-features
    cargo clippy --target wasm32-unknown-unknown --features hydrate --no-default-features

# Kill stale server processes on port 3000
_kill-stale:
    -lsof -i :3000 -t | xargs kill 2>/dev/null || true

# Run E2E tests via cargo-leptos + Playwright
e2e: _kill-stale db-reset db-seed
    SAMETE_TEST_MODE=true SAMETE_SMS_DRY_RUN=true cargo leptos end-to-end

# Run a single test by grep pattern — includes DB reset and rebuild.
# Only useful for tests that don't depend on prior DB state (e.g. block 1 tests).
# For dependent tests: run `just e2e` first, then target with `just e2e-single`.
e2e-single pattern: _kill-stale db-reset db-seed
    SAMETE_TEST_MODE=true SAMETE_SMS_DRY_RUN=true cargo leptos end-to-end -- --grep "{{pattern}}"

# Re-run E2E tests without resetting the DB — use when DB is already in correct state.
# Rebuilds the app. Kills any stale server on :3000 first.
e2e-rerun: _kill-stale
    SAMETE_TEST_MODE=true SAMETE_SMS_DRY_RUN=true cargo leptos end-to-end

# Build release
build:
    cargo leptos build --release

# Reset database (drop, create, migrate)
db-reset:
    sqlx database drop -y && sqlx database create && sqlx migrate run

# Seed test admin (for E2E)
db-seed:
    psql $DATABASE_URL -f seed/test_admin.sql

# Run pending migrations
db-migrate:
    sqlx migrate run

# Create a new migration
db-new name:
    sqlx migrate add {{name}}

# Generate sqlx offline query data
prepare:
    cargo sqlx prepare

# Format all code
fmt:
    cargo fmt --all

# Check formatting
fmt-check:
    cargo fmt --all -- --check

# Full validation (what CI runs)
check: fmt-check clippy test
