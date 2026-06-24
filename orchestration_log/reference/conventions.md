# Conventions

Last updated: 2026-06-22

## Model Tier Overrides

| Task type | Default tier | Override | Rationale |
|-----------|-------------|----------|-----------|
| POM/E2E fixes | sonnet (implementer) | — | Needs reasoning about race conditions, wait strategies |
| E2E investigation | sonnet (explore) | — | Needs to trace server function → SSR → Suspense chains |
| E2E debugging / suite flakiness investigation | sonnet (multiple parallel) | Required | History-informed; mandatory speculative-parallel per agentic-delegation. Single-agent investigation has historically missed cross-cutting causes (see Phase A 2026-04-20). |
| Pool metrics / binary analysis | haiku | — | Mechanical: run command, read output, report |
| CLAUDE.md / docs updates | opus (agent) | Required | Instructions for agents must be written by opus |
| Internet research | sonnet | — | Needs judgment to assess credibility of sources |
| leptos_config patching | sonnet (implementer) | — | Needs source-code-level reasoning about byte offsets |

**Lesson:** The user explicitly requires opus for anything touching agent instructions (CLAUDE.md, guidance docs). This is a hard override, not a suggestion. Traced to: session where sonnet agent was interrupted for docs work.

## Dispatch Rules

1. **E2E debugging is ALWAYS delegated.** Per `guidance/debugging-policy.md`. Orchestrator never reads screenshots, traces server function logic, or runs multiple E2E cycles.
2. **Long commands use `run_in_background` + redirect to file.** Never pipe through `| tail` or `| head`. Read output file separately.
3. **Source `.env.example` before any `just` target.** Docker Postgres requires `DATABASE_URL` set. Failure mode: `role "samete" does not exist` or `Connection refused`.
4. **Verify Postgres is Docker, not brew.** `docker compose ps` first. If brew Postgres is running on 5432, it shadows Docker's port mapping.

## Review Protocol

- spec-reviewer after implementer reports DONE (per dev-orchestration)
- code-quality-reviewer after spec passes
- For E2E POM changes: verification is running the suite, not code review. Run 2-3x to confirm non-flakiness.

## Forbidden Patterns

| Pattern | Why | Traced to |
|---------|-----|-----------|
| Claiming "all green" on single E2E run | Flakiness masks behind lucky runs | Session 2026-04-09: celebrated 58/58 on f8a3c41, but subsequent runs showed 1-3 failures |
| Pool config changes without metrics | Pool was never the bottleneck (max 4 connections) | Session 2026-04-09: $15 wasted on pool starvation hypothesis |
| `[patch.crates-io]` without running patched crate's own test suite | Byte-offset semantics broke silently | Session 2026-04-09: leptos_config regex patch broke config parsing |
| `waitForLoadState("load")` in POM | Waits for 14MB WASM download, intermittently exceeds timeout | Session 2026-04-09: every POM method with `"load"` eventually caused a timeout |
| `waitForLoadState("domcontentloaded")` after redirects in POM | Racy: if DOMContentLoaded fires before the call, it waits for the NEXT one (which never comes) → 30s timeout. Use `expect(page).toHaveURL()` / `not.toHaveURL()` (auto-retrying, race-free) instead. | Session 2026-06-20: 4 failed fix attempts before root-causing the race. Affects login, logout, completeOnboarding, selfRegister, advanceSeason. |
| `waitForURL` with `waitUntil: 'domcontentloaded'` | Same race as standalone `waitForLoadState` — the `waitUntil` option has identical semantics. | Session 2026-06-20: attempted as fix, failed identically. |
| Adding `value=""` to Leptos ActionForm inputs for hydration stability | Leptos hydration resets `.value` to match the attribute — erases Playwright-filled values. Absence of `value` attribute is better: Leptos has nothing to restore. | Session 2026-06-20: onboarding branch input. Added `value=""`, tests failed worse. Reverted. |
| Scapegoating the machine for test failures | Dev-mode E2E failures are real bugs, not environmental noise. Fix them. | Session 2026-06-20: user correction. |
| Running `brew services start postgresql` | Shadows Docker Postgres on port 5432 | Session 2026-04-09: 3 failed E2E runs from brew/Docker collision |
| Router-wide `tokio::time::timeout` middleware on Axum SSR routes | Drops the SSR future mid-render; Leptos Suspense never resolves; client hangs until its own navigation timeout. Identical-budget timeouts on both server and client maximize the failure surface. | Session 2026-04-20: commit `3ad9b65` reverted in commit `1f4df2c` after CI exposed it. |
| Orchestrator reading/editing `.rs` files or using Edit tool on source | Orchestrator context is the most expensive resource. Source file reads and edits are agent work. When debug agents fail, dispatch narrower agents — never intervene directly. | Session 2026-04-29: orchestrator read login.rs 10+ times and used Edit 8+ times to debug hydration/cookie issues. |
| `#[cfg(feature = "ssr")]` for values that must match across hydration | SSR and WASM branches produce different initial values, causing hydration mismatch. Use query params, shared constants, or Resource (with stable fallback) instead. | Session 2026-04-29: `is_pending` was true on SSR, false on client — invite-code-step disappeared during hydration. |
| ActionForm for server fns that set HttpOnly cookies | Fetch API's Set-Cookie handling differs from native POST. Cookies set via `ResponseOptions` may not apply to ActionForm fetch responses. Use native `<form method="post">` instead. | Session 2026-04-29: register_with_code session cookie never set via ActionForm; switched to native POST. |
| LEFT JOIN columns without `"column?"` suffix in sqlx query_as! | sqlx's offline cache may mark LEFT JOIN columns as non-nullable. At runtime, NULL values cause decode errors. Always use `"column?"` to force nullable decode. | Session 2026-04-29: list_invite_codes failed with "unexpected null; try decoding as Option" for redeemer_name. |
| `toBeEnabled()` guards on `goHome()`/`goToDashboard()` | When WASM fails to init (intermittent dev-mode), explicit `toBeEnabled` waits 15-30s then hard-fails. Playwright's `click()` auto-retry is more resilient — it retries continuously with backoff. | Session 2026-06-22: goHome logout-button 30s wait caused worse flakiness than the original `<main>` visible check. |
| Removing `page.goto("/admin")` after admin login in tests | Login redirect to /admin is not deterministic — SSR Suspense may not resolve admin state before the test proceeds. Explicit goto forces fresh SSR round-trip. | Session 2026-06-22: visual-audit test 207 failed consistently after goto removal. Restored. |
| `cargo sqlx prepare` without `--features ssr` | Queries exist only in the SSR feature gate. Bare prepare finds 0 queries, deletes the entire cache. Always: `cargo sqlx prepare --workspace -- --features ssr` | Session 2026-06-22: empty .sqlx/ after prepare. |
| Stripping `content-encoding` from cached-context.ts headers | Hypothesis was wrong — Chromium handles the Content-Encoding: br mismatch on route.fulfill() more gracefully than receiving raw bytes with no encoding header. The "fix" made hydration worse. | Session 2026-06-22: 4 consecutive failures after applying the encoding fix. Reverted. |

## Test Philosophy

- `just e2e` runs release binary (471KB WASM). `just e2e-dev` for dev mode (14MB, flaky).
- E2E tests are serial (shared DB state). Independent blocks (Epic 6, Logout) run separately.
- Static assets cached across tests via `cached-context.ts` fixture. Import `test`/`expect` from there, not `@playwright/test`.
- Pre-compressed WASM served via `precompress-and-test.sh`. `CompressionLayer` skips already-encoded responses.
- Verify with 3 consecutive green runs before declaring stability.
- `waitForLoadState` is banned in all forms after redirects. Use auto-retrying `toHaveURL` / `not.toHaveURL` assertions.
- Every review chain runs sequentially within a unit (spec → quality). Multiple units' chains can run in parallel.

- E2E tests are serial (shared DB state). Independent blocks (Epic 6, Logout) run separately.
- Static assets cached across tests via `cached-context.ts` fixture. Import `test`/`expect` from there, not `@playwright/test`.
- Pre-compressed WASM served via `precompress-and-test.sh`. `CompressionLayer` skips already-encoded responses.
- Verify with 3 consecutive green runs on CI before declaring stability. Dev-mode green is necessary but not sufficient.
- `waitForLoadState` is banned in all forms after redirects. Use auto-retrying `toHaveURL` / `not.toHaveURL` assertions.
- Every review chain runs sequentially within a unit (spec → quality). Multiple units' chains can run in parallel.
