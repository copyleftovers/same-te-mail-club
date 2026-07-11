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

# Run E2E tests against the release binary (471KB brotli WASM — CI-stable)
e2e: e2e-release

# Run E2E tests in dev mode — only for debugging; 14MB WASM may intermittently fail
e2e-dev: _kill-stale db-reset db-seed
    SAMETE_TEST_MODE=true SAMETE_SMS_DRY_RUN=true cargo leptos end-to-end

# Run a single test by grep pattern against the release binary — includes DB reset and rebuild.
# Only useful for tests that don't depend on prior DB state (e.g. block 1 tests).
# For dependent tests: run `just e2e` first, then target with `just e2e-single`.
e2e-single pattern: _kill-stale db-reset db-seed
    SAMETE_TEST_MODE=true SAMETE_SMS_DRY_RUN=true cargo leptos end-to-end --release -- --grep "{{pattern}}"

# Re-run E2E tests without resetting the DB — use when DB is already in correct state.
# Rebuilds the release binary. Kills any stale server on :3000 first.
e2e-rerun: _kill-stale
    SAMETE_TEST_MODE=true SAMETE_SMS_DRY_RUN=true cargo leptos end-to-end --release

# Build release with pre-compressed static assets
build:
    cargo leptos build --release
    @just _precompress --best

# Serve release build
serve: build
    ./target/release/samete

# Run E2E tests against release build
e2e-release: _kill-stale db-reset db-seed
    SAMETE_TEST_MODE=true SAMETE_SMS_DRY_RUN=true cargo leptos end-to-end --release

# Pre-compress static assets (quality: --best for release, -q 5 for dev)
_precompress quality="--best":
    @for f in target/site/pkg/*.wasm target/site/pkg/*.js target/site/pkg/*.css; do \
        [ -f "$$f" ] || continue; \
        brotli {{quality}} --keep --force "$$f"; \
        gzip --keep --force "$$f"; \
    done

# Reset database (drop, create, migrate)
# Force-disconnects all external clients (DataGrip, psql) before dropping so the drop is unconditional.
db-reset:
    docker compose exec -T db psql -U samete -d postgres -c "select pg_terminate_backend(pid) from pg_stat_activity where datname='samete' and pid <> pg_backend_pid();" || true
    sqlx database drop -y && sqlx database create && sqlx migrate run

# Seed test admin (for E2E)
db-seed:
    [ -n "${DATABASE_URL:-}" ] || { echo "ERROR: DATABASE_URL not set -- source .env.example"; exit 1; }
    psql $DATABASE_URL -f seed/test_admin.sql

# Run pending migrations
db-migrate:
    sqlx migrate run

# Create a new migration
db-new name:
    sqlx migrate add {{name}}

# Isolated capture: own free port + own sibling DB (samete_<suffix>).
# Never touches :3000 or the `samete` DB. Safe to run while dev server is up.
# mode: visual (default) = visual-audit screenshots only; full = full regression.
capture-isolated suffix mode="visual":
    bash scripts/isolated-capture.sh {{suffix}} {{mode}}

# Generate sqlx offline query data
prepare:
    cargo sqlx prepare --workspace -- --features ssr

# Format all code
fmt:
    cargo fmt --all

# Check formatting
fmt-check:
    cargo fmt --all -- --check

# Full validation (what CI runs)
check: fmt-check clippy test
