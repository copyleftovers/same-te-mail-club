# Codebase State

Last updated: 2026-04-20

## Module Inventory

| Module | Path | Purpose | Tests | Status |
|--------|------|---------|-------|--------|
| App shell | `src/app.rs` | Routes, guards, current_user Resource | E2E | Stable |
| Auth | `src/auth.rs` | Session, OTP, require_auth | E2E | Stable |
| Login | `src/pages/login.rs` | OTP flow, verify, logout | E2E | Stable |
| Onboarding | `src/pages/onboarding.rs` | Nova Poshta address | E2E | Stable |
| Home | `src/pages/home.rs` | Participant dashboard, season states | E2E | Stable |
| Admin: Dashboard | `src/admin/dashboard.rs` | Overview stats | E2E | Stable |
| Admin: Season | `src/admin/season.rs` | Create, launch, advance, cancel | E2E | Stable |
| Admin: Participants | `src/admin/participants.rs` | Register, deactivate | E2E | Stable |
| Admin: Assignments | `src/admin/assignments.rs` | Generate, swap, release | E2E | Stable |
| Admin: SMS | `src/admin/sms.rs` | Trigger SMS notifications | E2E | Stable |
| DB | `src/db.rs` | Pool creation, migrations | — | Stable (defaults) |
| Config | `src/config.rs` | Env-based config | — | Stable |
| Types | `src/types.rs` | Domain types, enums | — | Stable |
| Phone | `src/phone.rs` | E.164 normalization | Unit | Stable |

## E2E Test Suite

- **Total:** 58 tests across 3 serial blocks
- **Pass rate:** ~~58/58 (verified 2026-04-10)~~ — superseded 2026-04-20: prior claim was based on a single run (against project conventions). UNSTABLE as of 2026-04-20 — fresh 3-run verification returned 0/3 green; failures are SSR/hydration timeouts at 30s on `/login`, post-login redirects, and `/admin/*` pages. Last reliably-stable date unknown; D7 tracks investigation. Suite ran 58 tests; structure preserved.
- **Runtime:** 54.8s (dev), 18.2s (release) — based on green runs; current runtime indeterminate
- **Structure:** Main lifecycle chain (53 tests) + Account Management (3) + Session Management (2)
- **Fixture:** `cached-context.ts` caches WASM/JS/CSS/fonts across tests
- **Pre-compression:** `precompress-and-test.sh` runs before every E2E

## WASM

- Dev: 14MB raw, 1.9MB brotli (pre-compressed)
- Release: 1.87MB raw, 471KB brotli
- Profile: `opt-level = 'z'`, `lto = true`, `codegen-units = 1`
- wasm-opt: automatic via cargo-leptos 0.3.2 (`-Oz`)
- Optimization floor reached. Remaining gains: `build-std` (nightly) or `--split` (code splitting).

## Infrastructure

- Postgres 16 in Docker (`docker-compose.yml`)
- cargo-leptos 0.3.2
- Leptos 0.8.17, Axum 0.8.8
- macOS linker: classic (`-Wl,-ld_classic`) due to Apple ld assertion bug with thin LTO
- GitHub Actions CI: `.github/workflows/ci.yml` — `check` job + `e2e` job with Postgres service

## SSR Timeout

- REMOVED 2026-04-20 — the router-wide `tokio::time::timeout` middleware was incompatible with Leptos Suspense streaming (futures dropped mid-render leave Suspense boundaries unresolved, identical to Playwright's own 30s navigation timeout). D1 reopened. Future approach: per-Resource timeouts inside Suspense rather than router-level middleware.

## Known Limitations

1. Leptos SSR has no timeout — D1 reopened 2026-04-20 (prior 30s middleware reverted, see SSR Timeout section)
2. `leptos_config` pulls `regex` into WASM dependency tree (LTO eliminates it, but compilation is slower)
3. Docker Postgres adds latency vs native — may contribute to marginal E2E flakiness
4. `CompressionLayer` still re-compresses SSR HTML on the fly (small, fast — not a bottleneck)
5. ~~No CI pipeline~~ — RESOLVED: GitHub Actions pipeline added
6. E2E suite UNSTABLE as of 2026-04-20 — D7 tracks SSR/hydration timeout investigation
7. CI runs `check` job only as of 2026-04-20 — D8 tracks re-adding `e2e` job once D7 closes

## Next Actions (Priority Order)

1. Close D7: root-cause + fix the E2E suite SSR/hydration flakiness (Phase A in progress)
2. Close D8: re-add `e2e` job to CI workflow once D7 closes and 3 consecutive green runs are demonstrated
3. File upstream issue: `leptos_config` regex dependency (low priority — LTO handles it)
