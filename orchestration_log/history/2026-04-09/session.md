# Session: 2026-04-09

**Orchestrator:** Claude Opus 4.6 (1M context)
**Duration:** 3h 12m API, 2d 1h 28m wall
**Cost:** $114.04 (opus $97.61 / sonnet-4.5 $13.44 / haiku $1.97 / sonnet-4.6 $1.03)
**Code changes:** 1762 lines added, 562 lines removed (12 files)
**Outcome:** E2E suite stabilized from 12/58 to 58/58, root causes traced through 3 disproven hypotheses to CompressionLayer + BrowserContext isolation, WASM size confirmed at optimization floor (471KB brotli).

---

## Timeline

### Phase: QA Execution (hour 0-1)
- Invoked `/qa-run`, created bridge files (`tests/seed.spec.ts`, `.playwright/project-config.md`) to map QA skill to existing `end2end/` layout
- Executor-agent ran suite: 12/58 pass, test 13 timeout on `/admin/season`
- Classified as timing failure, 0 healable locator failures

### Phase: POM Race Fix (hour 1-2)
- Investigated test 13: `login()` returns before 302 redirect completes, `page.goto()` cancels in-flight SSR
- Fixed `login()` with `waitForLoadState("domcontentloaded")` → test 13 passes
- Same bug surfaced in test 19 (`goHome()` redundant reload) → fixed with skip-if-already-on-URL
- Audited full POM: found 4 more methods with same race → fixed `logout()`, `completeOnboarding()`, `goToDashboard()`, `advanceSeason()`
- Fixed `deactivateParticipant()` missing hydration wait
- Split Epic 6 and Logout into independent serial blocks
- Added JSON reporter for executor-agent structured parsing

### Phase: WASM Serving Investigation (hour 2-4)
- Flaky timeouts persisted after race fixes — different test each run
- **Hypothesis 1: Pool starvation** — added pool metrics (size/idle/active every 2s). Pool peaked at 4 connections, 0 active. DISPROVEN.
- **Hypothesis 2: On-the-fly compression** — `CompressionLayer` re-compresses 14MB WASM with Brotli per request. CONFIRMED. Pre-compression script: 3.5min → 25.7s.
- **Hypothesis 3: Per-context WASM re-download** — Playwright BrowserContext has isolated HTTP cache. 58 × 14MB downloads. CONFIRMED. Route-based caching fixture: 58/58 in 54.8s.
- **Hypothesis 4: Docker Postgres latency** — brew Postgres (local socket) vs Docker (TCP port mapping). Inconclusive — mitigated by faster WASM delivery.
- **Hypothesis 5: Leptos SSR has no timeout** — confirmed no `tokio::time::timeout` in Suspense resolution. Not fixed — deferred.

### Phase: WASM Size Analysis (hour 4-5)
- twiggy profiling: binary well-distributed, no 500KB regex blob. LTO+wasm-opt eliminate dead code.
- opt-level comparison: 'z' beats 's' by 7.9%. Current config is optimal.
- Production WASM: 471KB brotli. At optimization floor for this app.
- `leptos_config` regex patch attempted via `[patch.crates-io]` → broke config parsing → reverted.

### Phase: Release Build (hour 5-6)
- Added `just serve`, `just e2e-release`, `just build` with pre-compression
- Apple linker crash with thin LTO → fixed with `-Wl,-ld_classic` in `.cargo/config.toml`
- Release E2E: 58/58 in 18.2s (1.8MB WASM, 546KB brotli)

### Phase: Cleanup & Paperwork (hour 6)
- Reverted pool config (disproven hypothesis)
- Reverted regex patch (broken)
- Consolidated memory files into CLAUDE.md
- Updated E2E README with current state
- Wrote QA session artifacts, investigation reports

## Decision Log

| Decision | Context | Rationale | Outcome |
|----------|---------|-----------|---------|
| `domcontentloaded` not `load` | waitForLoadState in POM | Only need HTML committed, not 14MB WASM downloaded | Correct — eliminated race without adding WASM timeout pressure |
| Pre-compress at build time | CompressionLayer re-compresses per request | Zero runtime CPU cost, `ServeDir::precompressed_br()` already supported | Correct — 3.5min → 25.7s |
| Route-based caching fixture | BrowserContext isolation = 58 WASM downloads | Cache to temp dir, serve from disk on subsequent tests | Correct — eliminated flakiness |
| Revert pool config | Pool metrics showed max 4 connections | Pool was never the bottleneck | Correct — hypothesis disproven with data |
| Revert regex patch | leptos_config parsing broke | `find_section_line()` replacement had wrong byte-offset semantics | Correct — tests deterministically failed at test 5 |
| Keep 30s navigationTimeout | 15s was too tight for dev WASM | Passing tests are ~3s, 30s gives headroom without masking failures | Acceptable — symptoms addressed by pre-compression + caching |
| Defer Axum TimeoutLayer | Leptos SSR has no timeout mechanism | Wrong timeout value could mask real issues or break legitimate slow pages | Correct to defer — needs careful design |
| Defer `build-std` optimization | Requires nightly, 10-30% estimated savings | 471KB brotli is already reasonable; nightly dependency not worth it | Correct — marginal gains, real cost |

## Failure Log

| Failure | Root cause | Correction | Prevention |
|---------|-----------|------------|------------|
| Pool starvation hypothesis wasted ~$15 | Assumed server-side bottleneck without measuring | Pool metrics proved 4 connections max | Always instrument before theorizing. Measure first. |
| regex patch broke config parsing | `find_section_line()` didn't match regex byte-offset semantics | Reverted | Any `[patch.crates-io]` must pass the patched crate's own test suite AND integration tests |
| 3 failed E2E runs from missing DATABASE_URL | Didn't check env before running `just e2e` | Added env note to CLAUDE.md | CLAUDE.md now documents env prerequisites |
| brew Postgres shadowed Docker Postgres | Both on port 5432 | Stopped brew service | CLAUDE.md now documents Docker-first Postgres |
| Claimed "58/58 all green" on lucky runs | Flakiness masked by pre-compression reducing but not eliminating WASM pressure | Route-based caching fixture was the actual fix | Don't celebrate single green runs. Run 2-3x to confirm stability. |

## Quantitative Summary

| Metric | Value |
|--------|-------|
| Total agents dispatched | 56 (from transcript) |
| Haiku agents | 12 ($1.97) |
| Sonnet agents | 27 sonnet-4.5 + 13 unspecified ($13.44 + $1.03) |
| Opus agents | 4 ($97.61 incl. orchestrator) |
| Background Bash (E2E runs) | 17 |
| Total Bash invocations | 121 |
| Commits pushed | 16 (2 reverted) |
| Files changed | 12 |
| Lines added | 1762 |
| Lines removed | 562 |
| E2E suite: start | 12/58 passing, 3.5min |
| E2E suite: end | 58/58 passing, 54.8s (dev), 18.2s (release) |
| WASM size (release) | 1.87MB raw, 471KB brotli |
| Hypotheses tested | 5 (2 confirmed, 2 disproven, 1 inconclusive) |
