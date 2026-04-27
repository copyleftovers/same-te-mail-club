# Deferred Items

Last updated: 2026-04-20

## From E2E Investigation (2026-04-09)

| ID | Severity | Description | File | Status | Why deferred |
|----|----------|-------------|------|--------|-------------|
| D1 | Low | Leptos SSR has no timeout (resolution RETRACTED 2026-04-20: router-wide axum middleware was incompatible with Leptos Suspense streaming and was reverted) | `src/main.rs` | Open | No production incident; project relies on per-Resource discipline. PROPER fix: per-Resource timeouts inside Suspense rather than router-level middleware. |
| D2 | Low | `leptos_config` pulls `regex` into WASM dependency tree | upstream | Open | LTO+wasm-opt eliminate dead code. File upstream issue when ready. |
| D3 | Low | Docker Postgres latency may contribute to marginal E2E flakiness | infra | Open | Mitigated by pre-compression + caching. Last run was 58/58 clean. |
| D4 | ~~Low~~ | ~~No CI pipeline~~ | — | **RESOLVED 2026-04-10** | GitHub Actions added |
| D5 | Info | `build-std` (nightly) could reduce WASM by 10-30% | `Cargo.toml` | Open | 471KB brotli is already reasonable. Nightly dependency not justified. |
| D6 | Info | Code splitting (`--split`) could reduce initial load | `justfile` | Open | Doesn't reduce total bytes. Worth considering when app grows beyond 15 screens. |
| D7 | High | E2E suite flakiness — SSR/hydration timeouts at 30s on `/login`, `/`, `/admin/*` pages under sustained load | `end2end/tests/` | Open | Investigation underway in Phase A of session 2026-04-20. Phase B history dive ruled out pool starvation (peaks at 4 of 10) and WASM compression (already fixed); residual root cause not yet identified. |
| D8 | Medium | E2E job removed from CI workflow until suite stability is hardened | `.github/workflows/ci.yml` | Open | Per user direction in session 2026-04-20, CI was downscoped to `check` job (lint + unit tests) only. Re-add `e2e` job once D7 is closed and 3 consecutive green local runs are demonstrated. |
