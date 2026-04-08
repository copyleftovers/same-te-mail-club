#!/usr/bin/env bash
set -euo pipefail

# Pre-compress large static assets so ServeDir serves .br files directly.
# CompressionLayer skips responses that already have Content-Encoding.
# Quality 5 balances speed (~150ms) vs ratio (14MB -> 1.9MB).
#
# cargo-leptos runs end2end-cmd from the end2end-dir, so paths to
# build artifacts use .. to reach the project root.
for f in ../target/site/pkg/*.wasm ../target/site/pkg/*.js ../target/site/pkg/*.css; do
    [ -f "$f" ] || continue
    brotli -q 5 --keep --force "$f"
    gzip -5 --keep --force "$f"
done

exec npx playwright test "$@"
