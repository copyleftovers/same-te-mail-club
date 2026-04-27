# WASM Serving Stalls — Investigation Report

Date: 2026-04-08 to 2026-04-09

## The Question
Why do E2E tests intermittently timeout on WASM hydration (buttons stay disabled, `domcontentloaded` never fires)?

## Hypotheses Tested

### 1. Connection Pool Starvation — DISPROVEN
**Method:** Added pool metrics (size, idle, active) every 2s during E2E run.
**Evidence:** Pool peaked at 4 connections, 0 active at every sample point. Never reached max (10).
**Verdict:** Pool is never contended. sqlx defaults (10 connections) are more than sufficient.

### 2. On-the-fly WASM Compression — CONFIRMED
**Method:** Checked `CompressionLayer` behavior in tower-http source. Verified no pre-compressed files existed.
**Evidence:** `CompressionLayer` re-compresses 14MB WASM with Brotli on every request. No `.br` files alongside `.wasm`. Single Axum event loop handles both compression and SSR.
**Fix:** Pre-compress at build time via `end2end/precompress-and-test.sh`. `ServeDir::precompressed_br()` serves `.br` directly.
**Result:** Suite runtime dropped from 3.5min to 25.7s.

### 3. Per-Context WASM Re-Download — CONFIRMED
**Method:** Verified Playwright BrowserContext isolation (docs + issue #22865). Each context has empty HTTP cache.
**Evidence:** 58 tests × fresh context = 58 × 14MB WASM downloads. No cache sharing.
**Fix:** Route-based caching fixture (`cached-context.ts`) — intercepts static assets, caches to temp dir, serves from cache.
**Result:** 58/58 passing in 54.8s.

### 4. Docker Postgres Latency — UNRESOLVED
**Method:** Compared brew Postgres (local socket) vs Docker Postgres (TCP port mapping).
**Evidence:** All-green runs happened on brew Postgres. Docker runs showed occasional timeouts. But insufficient controlled testing to isolate as sole cause.
**Status:** Mitigated by faster WASM delivery (less total server load), not directly fixed.

### 5. Leptos SSR Has No Timeout — CONFIRMED (not fixed)
**Method:** Searched leptos source for timeout mechanisms in Suspense rendering.
**Evidence:** No `tokio::time::timeout` wrapping SSR Resource resolution. If a DB query hangs, the HTTP response hangs indefinitely.
**Fix proposed:** `TimeoutLayer` on Axum router. Deferred — needs careful timeout value selection.

## Artifacts
- Pool metrics code: `src/main.rs` (conditional on `SAMETE_LOG_POOL=true`)
- Pre-compression: `end2end/precompress-and-test.sh`
- Caching fixture: `end2end/tests/fixtures/cached-context.ts`
