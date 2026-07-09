#!/usr/bin/env bash
# Parallel-safe isolated capture harness.
#
# Runs a worktree's release binary on its OWN free port against its OWN sibling
# DB (samete_<suffix>) on the live Postgres instance. Tears down ONLY what it
# creates — never touches :3000, never touches the `samete` DB, never kills
# foreign processes.
#
# Usage (run from the worktree root you want to capture):
#   scripts/isolated-capture.sh <suffix> [visual|full]
#
#   suffix  — unique identifier for this run; sanitized to [a-z0-9_].
#             Becomes the DB name suffix: samete_<suffix>.
#   mode    — visual (default): runs tests/visual-audit.spec.ts only.
#             full: runs the complete Playwright suite.
#
# Prerequisites: cargo leptos, sqlx, psql, brotli, gzip, python3, curl, npx.
# Postgres must be reachable at localhost:5432 with role `samete` as owner.
set -euo pipefail

# --- arguments -----------------------------------------------------------

SUFFIX_RAW="${1:?usage: scripts/isolated-capture.sh <suffix> [visual|full]}"
CAPTURE_MODE="${2:-visual}"

# --- sanitize suffix: lowercase, replace non-[a-z0-9_] with underscore ---

SUFFIX="$(printf '%s' "$SUFFIX_RAW" | tr '[:upper:]' '[:lower:]' | tr -c 'a-z0-9_' '_')"
[ -n "$SUFFIX" ] || { echo "FATAL: suffix is empty after sanitization"; exit 1; }

# --- GUARD: refuse to operate on the dev DB or dev port ------------------

SIBLING_DBNAME="samete_${SUFFIX}"
[ "$SIBLING_DBNAME" != "samete" ] \
    || { echo "FATAL: computed DB name is 'samete' — refusing to operate on the dev DB"; exit 1; }

SIBLING_DBURL="postgres://samete:samete@localhost:5432/${SIBLING_DBNAME}"

# --- allocate a free OS port (port-0 trick — race-REDUCED, not race-free) ---
# TOCTOU window: the probe socket is closed before the server binds, so another
# process can claim the port in between. The window is small; the server fails
# loud on EADDRINUSE (caught by the ready-wait below → rerun).

ISOLATED_PORT="$(python3 -c \
    'import socket; s=socket.socket(); s.bind(("127.0.0.1",0)); print(s.getsockname()[1]); s.close()')"
[ "$ISOLATED_PORT" != "3000" ] \
    || { echo "FATAL: OS allocated port 3000 — refusing to bind the dev port; retry"; exit 1; }

# --- teardown: kills only our spawned PID, drops only our sibling DB ----

SERVER_PID=""
teardown() {
    [ -n "$SERVER_PID" ] && kill "$SERVER_PID" 2>/dev/null || true
    DATABASE_URL="$SIBLING_DBURL" sqlx database drop -y >/dev/null 2>&1 || true
}
trap teardown EXIT INT TERM

echo "[isolated-capture] suffix=${SUFFIX} db=${SIBLING_DBNAME} port=${ISOLATED_PORT} mode=${CAPTURE_MODE}"

# --- DB lifecycle: create sibling, migrate, seed -------------------------

DATABASE_URL="$SIBLING_DBURL" sqlx database create
DATABASE_URL="$SIBLING_DBURL" sqlx migrate run
# seed/test_admin.sql uses ON CONFLICT DO NOTHING — safe to run multiple times
psql "$SIBLING_DBURL" -f seed/test_admin.sql

# --- build release binary + pre-compress static assets ------------------

cargo leptos build --release

for asset in target/site/pkg/*.wasm target/site/pkg/*.js target/site/pkg/*.css; do
    [ -f "$asset" ] || continue
    brotli -q 5 --keep --force "$asset"
    gzip -5 --keep --force "$asset"
done

# --- serve binary directly on the isolated port + DB --------------------

LEPTOS_SITE_ADDR="127.0.0.1:${ISOLATED_PORT}" \
LEPTOS_SITE_ROOT="target/site" \
LEPTOS_SITE_PKG_DIR="pkg" \
LEPTOS_OUTPUT_NAME="samete" \
DATABASE_URL="$SIBLING_DBURL" \
SAMETE_TEST_MODE=true \
SAMETE_SMS_DRY_RUN=true \
    ./target/release/samete &
SERVER_PID=$!

# --- wait for server ready: poll until 200 or process death (max 60s) ---

wait_for_server_ready() {
    local attempts=120
    local attempt
    for attempt in $(seq 1 "$attempts"); do
        if curl -sf "http://127.0.0.1:${ISOLATED_PORT}/" >/dev/null 2>&1; then
            echo "[isolated-capture] server ready after ${attempt} probes"
            return 0
        fi
        if ! kill -0 "$SERVER_PID" 2>/dev/null; then
            echo "FATAL: server process ${SERVER_PID} exited before becoming ready"
            return 1
        fi
        sleep 0.5
    done
    echo "FATAL: server did not become ready within $((attempts / 2))s"
    return 1
}
wait_for_server_ready

# --- run Playwright against the isolated endpoint -----------------------

cd end2end
[ -d node_modules ] || npm ci

if [ "$CAPTURE_MODE" = "full" ]; then
    CAPTURE_BASE_URL="http://127.0.0.1:${ISOLATED_PORT}" \
    DATABASE_URL="$SIBLING_DBURL" \
        npx playwright test
else
    CAPTURE_BASE_URL="http://127.0.0.1:${ISOLATED_PORT}" \
    DATABASE_URL="$SIBLING_DBURL" \
        npx playwright test tests/visual-audit.spec.ts
fi
