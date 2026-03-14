# samete development commands

default:
    @just --list

# Start dev server with hot reload
dev:
    cargo leptos watch

# Run unit tests
test:
    cargo test

# Run clippy on all targets
clippy:
    cargo clippy --all-targets

# Run E2E tests via cargo-leptos + Playwright
e2e: db-reset db-seed
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
