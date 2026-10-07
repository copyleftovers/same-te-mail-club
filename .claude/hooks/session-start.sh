#!/usr/bin/env bash
# Claude Code cloud sessions only: install the pinned toolchain so `just e2e-release`
# works in a fresh container. Synchronous on purpose: no agent turn may race a
# half-installed toolchain. Idempotent; no secrets.
set -euo pipefail
[ "${CLAUDE_CODE_REMOTE:-}" = "true" ] || exit 0
cd "$CLAUDE_PROJECT_DIR"
# The exports do not depend on the bootstrap result, so write them first: a network
# failure during install must still leave PATH and DATABASE_URL set for the session.
{
    echo "export PATH=\"$CLAUDE_PROJECT_DIR/.tools/bin:\$HOME/.cargo/bin:\$PATH\""
    echo "export PLAYWRIGHT_BROWSERS_PATH=\"${PLAYWRIGHT_BROWSERS_PATH:-$HOME/.cache/ms-playwright}\""
    echo 'export DATABASE_URL="postgres://samete:samete@localhost/samete"'
} >> "$CLAUDE_ENV_FILE"
bash scripts/bootstrap-toolchain.sh >&2 \
    || { echo "[session-start] bootstrap failed; see output above" >&2; exit 1; }
