#!/usr/bin/env bash
# Idempotent toolchain bootstrap — the single home of every tool pin.
# Used by: CI (.github/workflows/ci.yml), the Claude Code cloud SessionStart hook
# (.claude/hooks/session-start.sh), and developers (README "Setup").
# Safe to re-run: every step checks before installing. No secrets.
set -euo pipefail
cd "$(dirname "$0")/.."

CARGO_BINSTALL_VERSION="1.25.1"
CARGO_LEPTOS_VERSION="0.3.7"
SQLX_CLI_VERSION="0.8.6"
JUST_VERSION="1.58.0"
WASM_OPT_VERSION="0.116.1"
CARGO_AUDIT_VERSION="0.22.2"
TAILWIND_VERSION="v4.2.1"
TAILWIND_SHA256_LINUX_X64="39e8d4e24b3c83b0a6e69e100a972fbc75d5fef8dce47b3ddac3cf92dea81fe3"
TAILWIND_SHA256_MACOS_ARM64="e510af7928750c9ee8d5ff2e5e98088bd5b99a8a8e2c554668621c7e151fa91f"
TAILWIND_SHA256_MACOS_X64="019e5cfa441992ede2772c6faaeb8d7fb1726aab50b1138c0aa38e88f4b7bd44"
POSTGRES_READY_TIMEOUT_SECONDS=60

log() { echo "[bootstrap] $*"; }
fatal() { echo "[bootstrap] FATAL: $*" >&2; exit 1; }

# macOS ships shasum, not sha256sum.
sha256_of() { # args: FILE
    if command -v sha256sum >/dev/null; then sha256sum "$1"; else shasum -a 256 "$1"; fi | cut -d' ' -f1
}
sha256_matches() { # args: EXPECTED_SHA FILE
    [ "$(sha256_of "$2" 2>/dev/null)" = "$1" ]
}

# --- Rust targets ---
rustup target list --installed | grep -qx wasm32-unknown-unknown || rustup target add wasm32-unknown-unknown

# --- cargo-binstall ---
if ! command -v cargo-binstall >/dev/null; then
    log "installing cargo-binstall $CARGO_BINSTALL_VERSION"
    cargo install --locked cargo-binstall --version "$CARGO_BINSTALL_VERSION"
elif ! cargo binstall -V 2>/dev/null | grep -qwF "$CARGO_BINSTALL_VERSION"; then
    log "installing cargo-binstall $CARGO_BINSTALL_VERSION"
    cargo binstall --no-confirm --locked "cargo-binstall@$CARGO_BINSTALL_VERSION"
else
    log "cargo-binstall $CARGO_BINSTALL_VERSION present"
fi

# --- pinned cargo tools: install only if missing or wrong version ---
ensure_tool() { # args: CRATE VERSION PROBE_CMD...
    local crate="$1" version="$2"; shift 2
    if "$@" 2>/dev/null | grep -qwF "$version"; then log "$crate $version present"; return; fi
    log "installing $crate $version"
    cargo binstall --no-confirm --locked "$crate@$version"
}
ensure_tool cargo-leptos "$CARGO_LEPTOS_VERSION" cargo leptos --version
ensure_tool sqlx-cli "$SQLX_CLI_VERSION" sqlx --version
ensure_tool just "$JUST_VERSION" just --version
ensure_tool cargo-audit "$CARGO_AUDIT_VERSION" cargo audit --version
# wasm-opt --version prints only the major ("version 116"), so the crate's minor/patch is not verifiable.
if wasm-opt --version 2>/dev/null | grep -qw "version 116"; then
    log "wasm-opt $WASM_OPT_VERSION present"
else
    log "installing wasm-opt $WASM_OPT_VERSION"
    cargo binstall --no-confirm --locked "wasm-opt@$WASM_OPT_VERSION"
fi

# --- tailwind: pinned binary on a repo-controlled PATH entry ---
mkdir -p .tools/bin
case "$(uname -s)-$(uname -m)" in
    Linux-x86_64)  tw_asset=tailwindcss-linux-x64;   tw_sha="$TAILWIND_SHA256_LINUX_X64" ;;
    Darwin-arm64)  tw_asset=tailwindcss-macos-arm64; tw_sha="$TAILWIND_SHA256_MACOS_ARM64" ;;
    Darwin-x86_64) tw_asset=tailwindcss-macos-x64;   tw_sha="$TAILWIND_SHA256_MACOS_X64" ;;
    *) fatal "no pinned tailwind for $(uname -s)-$(uname -m)" ;;
esac
if ! sha256_matches "$tw_sha" .tools/bin/tailwindcss; then
    log "installing tailwindcss $TAILWIND_VERSION ($tw_asset)"
    # Verify before the binary can appear at its PATH location.
    tw_download=.tools/bin/tailwindcss.download
    trap 'rm -f "$tw_download"' EXIT
    curl -fsSL -o "$tw_download" \
        "https://github.com/tailwindlabs/tailwindcss/releases/download/${TAILWIND_VERSION}/${tw_asset}"
    sha256_matches "$tw_sha" "$tw_download" || fatal "tailwindcss sha256 mismatch"
    chmod +x "$tw_download"
    mv "$tw_download" .tools/bin/tailwindcss
else
    log "tailwindcss $TAILWIND_VERSION present"
fi

# --- Node deps + the exact Playwright browser revision ---
node_stamp=end2end/node_modules/.bootstrap-lock-sha256
lock_sha="$(sha256_of end2end/package-lock.json)"
if [ "$(cat "$node_stamp" 2>/dev/null)" = "$lock_sha" ]; then
    log "node_modules matches package-lock.json"
else
    log "installing node dependencies (npm ci)"
    (cd end2end && npm ci)
    echo "$lock_sha" > "$node_stamp"
fi
pw_rev="$(node -e 'const b=require("./end2end/node_modules/playwright-core/browsers.json").browsers.find(x=>x.name==="chromium-headless-shell");console.log(b.revision)')"
pw_dir="${PLAYWRIGHT_BROWSERS_PATH:-$HOME/.cache/ms-playwright}"
if [ -f "$pw_dir/chromium_headless_shell-$pw_rev/INSTALLATION_COMPLETE" ]; then
    log "playwright chromium-headless-shell $pw_rev present in $pw_dir"
else
    log "installing playwright chromium $pw_rev into $pw_dir"
    (cd end2end && PLAYWRIGHT_BROWSERS_PATH="$pw_dir" npx playwright install chromium)
fi

# --- Postgres (role samete/samete on localhost:5432) ---
postgres_ready() { pg_isready -h localhost -p 5432 >/dev/null 2>&1; }

command -v pg_isready >/dev/null && command -v psql >/dev/null \
    || fatal "Postgres client tools (pg_isready, psql) are required; install the postgresql client package"
if ! postgres_ready; then
    if docker info >/dev/null 2>&1; then
        docker compose up -d db
    elif command -v pg_ctlcluster >/dev/null; then
        pg_ctlcluster "$(ls /usr/lib/postgresql | sort -n | tail -1)" main start
    else
        fatal "no Postgres on :5432 and neither docker nor pg_ctlcluster available"
    fi
    for _ in $(seq "$POSTGRES_READY_TIMEOUT_SECONDS"); do postgres_ready && break; sleep 1; done
    postgres_ready || fatal "Postgres not ready on :5432 after ${POSTGRES_READY_TIMEOUT_SECONDS}s"
fi
CREATE_ROLE_SQL="CREATE ROLE samete LOGIN CREATEDB PASSWORD 'samete'"
if ! psql "postgres://samete:samete@localhost:5432/postgres" -tAc 'select 1' >/dev/null 2>&1; then
    log "creating role samete"
    if [ "$(id -u)" = 0 ] && id postgres >/dev/null 2>&1; then
        su postgres -c "psql -c \"$CREATE_ROLE_SQL\""
    else
        fatal "role samete is unreachable and the postgres OS user is not available; run as a Postgres superuser: $CREATE_ROLE_SQL"
    fi
fi

log "done — export PATH=\"$PWD/.tools/bin:\$PATH\" PLAYWRIGHT_BROWSERS_PATH=\"$pw_dir\""
