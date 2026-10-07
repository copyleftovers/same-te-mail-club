#!/usr/bin/env bash
# Fail unless Playwright actually ran during this E2E invocation.
#
# WHY: `cargo leptos end-to-end` exits 0 when the cargo build fails and then never
# starts Playwright — CI was false-green from 2026-07-10 to 2026-07-25 with zero
# tests executed. Playwright's JSON reporter (end2end/playwright.config.ts) writes
# end2end/results.json on every run; a fresh file with >0 executed tests is the proof.
#
# Usage (from repo root): scripts/assert-playwright-ran.sh <start-marker>
#   <start-marker> — a file touched BEFORE the E2E command started.
set -euo pipefail

marker="${1:?usage: scripts/assert-playwright-ran.sh <start-marker>}"
results="end2end/results.json"

[ -f "$results" ] \
    || { echo "FATAL: $results missing — Playwright never ran (build failure?)"; exit 1; }
[ "$results" -nt "$marker" ] \
    || { echo "FATAL: $results predates this run — Playwright never ran (build failure?)"; exit 1; }

node -e '
const s = require(process.argv[1]).stats;
console.log(`playwright ran: expected=${s.expected} unexpected=${s.unexpected} flaky=${s.flaky} skipped=${s.skipped}`);
if (s.expected + s.unexpected + s.flaky === 0) {
    console.error("FATAL: Playwright executed 0 tests");
    process.exit(1);
}
if (s.unexpected > 0) {
    console.error("FATAL: Playwright reported failures");
    process.exit(1);
}
' "$PWD/$results"
