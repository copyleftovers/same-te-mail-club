#!/usr/bin/env bash
# Idempotent toolchain bootstrap — the single home of every tool pin.
# Used by: CI (.github/workflows/ci.yml), the Claude Code cloud SessionStart hook
# (.claude/hooks/session-start.sh), and developers (README "Setup").
# Safe to re-run: every step checks before installing. No secrets.
set -euo pipefail
cd "$(dirname "$0")/.."

CARGO_LEPTOS_VERSION="0.3.7"
SQLX_CLI_VERSION="0.8.6"
JUST_VERSION="1.58.0"
WASM_OPT_VERSION="0.116.1"
CARGO_AUDIT_VERSION="0.22.2"
TAILWIND_VERSION="v4.2.1"
TAILWIND_SHA256_LINUX_X64="39e8d4e24b3c83b0a6e69e100a972fbc75d5fef8dce47b3ddac3cf92dea81fe3"

log() { echo "[bootstrap] $*"; }

# --- Rust targets ---
rustup target list --installed | grep -qx wasm32-unknown-unknown || rustup target add wasm32-unknown-unknown

# --- cargo-binstall ---
command -v cargo-binstall >/dev/null || cargo install --locked cargo-binstall

# --- pinned cargo tools: install only if missing or wrong version ---
ensure_tool() { # args: CRATE VERSION PROBE_CMD...
    local crate="$1" version="$2"; shift 2
    if "$@" 2>/dev/null | grep -qF "$version"; then log "$crate $version present"; return; fi
    log "installing $crate $version"
    cargo binstall --no-confirm --locked "$crate@$version"
}
ensure_tool cargo-leptos "$CARGO_LEPTOS_VERSION" cargo leptos --version
ensure_tool sqlx-cli "$SQLX_CLI_VERSION" sqlx --version
ensure_tool just "$JUST_VERSION" just --version
ensure_tool cargo-audit "$CARGO_AUDIT_VERSION" cargo audit --version
# wasm-opt --version prints "version 116", not the crate version string.
if wasm-opt --version 2>/dev/null | grep -q "version 116"; then
    log "wasm-opt $WASM_OPT_VERSION present"
else
    log "installing wasm-opt $WASM_OPT_VERSION"
    cargo binstall --no-confirm --locked "wasm-opt@$WASM_OPT_VERSION"
fi

# --- tailwind: pinned binary on a repo-controlled PATH entry ---
mkdir -p .tools/bin
case "$(uname -s)-$(uname -m)" in
    Linux-x86_64) tw_asset=tailwindcss-linux-x64; tw_sha="$TAILWIND_SHA256_LINUX_X64" ;;
    *) echo "[bootstrap] FATAL: no pinned tailwind sha256 for $(uname -s)-$(uname -m)"; exit 1 ;;
esac
if ! echo "$tw_sha  .tools/bin/tailwindcss" | sha256sum -c --status 2>/dev/null; then
    log "installing tailwindcss $TAILWIND_VERSION ($tw_asset)"
    curl -fsSL -o .tools/bin/tailwindcss \
        "https://github.com/tailwindlabs/tailwindcss/releases/download/${TAILWIND_VERSION}/${tw_asset}"
    echo "$tw_sha  .tools/bin/tailwindcss" | sha256sum -c --status \
        || { echo "[bootstrap] FATAL: tailwindcss sha256 mismatch"; exit 1; }
    chmod +x .tools/bin/tailwindcss
else
    log "tailwindcss $TAILWIND_VERSION present"
fi

# --- Node deps + the exact Playwright browser revision ---
(cd end2end && npm ci)
pw_rev="$(node -e 'const b=require("./end2end/node_modules/playwright-core/browsers.json").browsers.find(x=>x.name==="chromium-headless-shell");console.log(b.revision)')"
pw_dir="${PLAYWRIGHT_BROWSERS_PATH:-$HOME/.cache/ms-playwright}"
if [ -f "$pw_dir/chromium_headless_shell-$pw_rev/INSTALLATION_COMPLETE" ]; then
    log "playwright chromium-headless-shell $pw_rev present in $pw_dir"
else
    log "installing playwright chromium $pw_rev into $pw_dir"
    (cd end2end && PLAYWRIGHT_BROWSERS_PATH="$pw_dir" npx playwright install chromium)
fi

# --- Postgres (role samete/samete on localhost:5432) ---
if ! pg_isready -h localhost -p 5432 >/dev/null 2>&1; then
    if docker info >/dev/null 2>&1; then
        docker compose up -d db
    elif command -v pg_ctlcluster >/dev/null; then
        pg_ctlcluster "$(ls /usr/lib/postgresql | sort -n | tail -1)" main start
    else
        echo "[bootstrap] FATAL: no Postgres on :5432 and neither docker nor pg_ctlcluster available"; exit 1
    fi
    until pg_isready -h localhost -p 5432 >/dev/null 2>&1; do sleep 1; done
fi
if ! psql "postgres://samete:samete@localhost:5432/postgres" -tAc 'select 1' >/dev/null 2>&1; then
    log "creating role samete"
    su postgres -c "psql -c \"CREATE ROLE samete LOGIN CREATEDB PASSWORD 'samete'\""
fi

log "done — export PATH=\"$PWD/.tools/bin:\$PATH\" PLAYWRIGHT_BROWSERS_PATH=\"$pw_dir\""
