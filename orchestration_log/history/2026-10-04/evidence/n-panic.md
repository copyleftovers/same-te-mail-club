# n-panic — reactive_graph disposal panic: root cause

Binding: stop-yapping / dry / first-principles / kiss, read in full. Raw logs in `logs/n-*.log`; this file holds verdicts.

## Verdict

- **Panic source:** `leptos_i18n-0.6.1/src/context.rs:213` — `Effect::new_isomorphic` (line 206) reads `locale_signal.get()` (RwSignal defined at `context.rs:307` via `provide_i18n_context_with_options_inner`). Reached from `src/app.rs:73` `crate::i18n::i18n::provide_i18n_context()`.
- **Mechanism:** on the server, `new_isomorphic` does not run the effect at once. It spawns a tokio task for the first run (`reactive_graph-0.2.13/src/effect/effect.rs:402-440`). That task always runs the closure on first poll (`first_run`), even when the owner has been disposed. `leptos_integration_utils-0.8.8/src/lib.rs:121-125` force-disposes the request Owner (`unset_with_forced_cleanup`) when the response stream ends. If the response finishes before the task is first polled, the signal read hits a disposed arena slot and the task panics.
- **Why it crashes the process:** `Cargo.toml` `[profile.release] panic = "abort"`. A panic inside a detached tokio task would be harmless under unwind (tokio catches it). Under abort it kills the whole server, and every later test fails with `ERR_CONNECTION_REFUSED`.
- **Not a client-disconnect bug.** It also reproduces with complete, non-aborted concurrent requests. It is not in project code, and it is not a toolchain or `recursion_limit` effect.
- **Upstream status:** `leptos_i18n` 0.6.2 (latest) has identical `init_context_inner` code. `<I18nContextProvider>` goes through the same `init_context_inner`, so switching to it does not fix anything. Leptos issues #3627 and #2842 show the same panic class (owner disposed before a spawned SSR task runs); both are open or closed without a fix.

## Panic evidence

A logs (`logs/f-e2e-run1.log:274`, `logs/f-e2e-run2.log:75`), release build, stripped, no frames:
```
thread 'tokio-rt-worker' (25508) panicked at .../reactive_graph-0.2.13/src/traits.rs:394:39:
Tried to access a reactive value that has already been disposed.
stack backtrace:
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.
```
After that, cargo-leptos's own `ProcessHandle should not have been dropped` panic (`f-e2e-run1.log:398`) is the harness tearing down after the server died. It is not a second bug.

B-setup repro, n-run2 (`logs/n-run2.log`):
```
thread 'tokio-rt-worker' (25768) panicked at .../reactive_graph-0.2.13/src/signal/subscriber_traits.rs:112:29:
Tried to access a reactive value that has already been disposed.
```
`subscriber_traits.rs:112` is `to_any_source`, inside the `track()` step of `.get()`. It fires when the signal is already disposed; `traits.rs:394` fires when the signal is disposed concurrently. Both are the same `.get()`-on-disposed path. Only the `:394` variant has been pinned to i18n by a debug-info backtrace. The `:112` variant is inferred to be the same, not observed with a location.

Diagnostic build (`RUSTFLAGS='--cfg leptos_debuginfo'`, `CARGO_PROFILE_RELEASE_STRIP=none`, `DEBUG=line-tables-only`), from the stress probe (`logs/n-stress-abort-debuginfo-backtrace.log`):
```
panicked at .../reactive_graph-0.2.13/src/traits.rs:394:39:
At .../leptos_i18n-0.6.1/src/context.rs:213:38, you tried to access a reactive value which was
defined at .../leptos_i18n-0.6.1/src/context.rs:307:19, but it has already been disposed.
   3: {closure#0}<RwSignal<samete::i18n::i18n::Locale, SyncStorage>>   reactive_graph/src/traits.rs:75
   5: get<RwSignal<samete::i18n::i18n::Locale, ..>>                     reactive_graph/src/traits.rs:394
   6: {closure#1}<samete::i18n::i18n::Locale>                         leptos_i18n-0.6.1/src/context.rs:213
   7: run<leptos_i18n::context::init_context_inner::{closure_env#1}>   reactive_graph/src/effect/effect_function.rs:19
   9: run_in_effect_scope<.. new_isomorphic ..>                        reactive_graph/src/effect/effect.rs:142
  15: {async_block#0}<.. new_isomorphic ..>                            reactive_graph/src/effect/effect.rs:429
  21: tokio::runtime::task::harness::poll_future                       tokio-1.50.0/.../harness.rs:535
```
8/8 stress panics show this same `At … context.rs:213:38` location: 7 restarts in the abort-request loop plus 1 in the full-request loop.

Request and route: not tied to one route. The panic is a detached task, so it is not attributed to any request. In E2E it fires at test boundaries, when a quick or redirecting response finishes:

| Run | Last pass before panic | Failing test |
|---|---|---|
| A run1 | `visual-audit:665` SMS report | `:676` home assignment |
| A run2 | `mail_club:236` non-admin → /admin redirect | `:251` 4.1 create season |
| n-run2 | `mail_club:217` self-register B | `:226` self-register C |
| n-e2e-path-run1 | `visual-audit:447` participants list | `:462` home no season |
| n-e2e-path-run2 | — | `visual-audit` cancelled season |

## A/B diff

| Axis | A (f-e2e) | B (U4 implementer) | Material? |
|---|---|---|---|
| Harness | `cargo leptos end-to-end --release` (cargo-leptos spawns the server, pipes its stdout, runs `precompress-and-test.sh`) | `scripts/isolated-capture.sh full` (runs the binary directly, stdout to a file) | **Yes: crash rate 4/5 vs 1/12** (Repro table) |
| rustc | 1.97.1 | 1.99.0 | No: A's harness on 1.99.0 still crashed 2/3 |
| recursion attr | `RUSTC_BOOTSTRAP=1 RUSTFLAGS=-Zcrate-attr=recursion_limit="1024"` (all crates) | `#![recursion_limit]` in src | No: same harness-on-B-code result. `rustc_version` channel detection is unaffected by `RUSTC_BOOTSTRAP` (leptos/tachys/reactive_graph `build.rs`) |
| cargo-leptos | 0.3.7 | 0.3.7 (installed) | No: identical |
| profile | release, `panic="abort"` | same | Same, and this is what makes it fatal |
| Playwright browser | `/opt/pw-browsers` (1194 binary under the 1208 name) | scratchpad `pwb` shim, now broken (points at a non-existent `/opt/.../1208/chrome-linux`); same 1194 binary | No. n-run0 failed on the broken shim before any request (`logs/n-run0-browserpath-FAIL.log`); n-runs used `/opt/pw-browsers` |

Hypotheses considered (max 3):
1. **Harness timing (supported by counts).** The race window is "stream end vs first poll of the spawned effect task". The cargo-leptos path widens it. One candidate mechanism, unverified: the server's synchronous tracing writes go to a cargo-leptos stdout pipe (tokio-process-tools `ReliableWithBackpressure`), which stalls worker threads and delays the effect task.
2. Toolchain or recursion attr. Rejected (rows above).
3. Client disconnect mid-stream. Rejected: the full-request stress also panics (`logs/n-stress-noabort.log`, round 25 of 100, 3.6 s).

## Repro

Worktree `1a3c8a5` + uncommitted `#![recursion_limit = "256"]` in `src/lib.rs` and `src/main.rs`; rustc 1.99.0, cargo-leptos 0.3.7, `PLAYWRIGHT_BROWSERS_PATH=/opt/pw-browsers`.

| Run | Harness / build | Result | Panic | Log |
|---|---|---|---|---|
| n-run0 | isolated, B's `pwb` path | browser launch fail, 0 requests | — | `n-run0-browserpath-FAIL.log` |
| n-run1 | isolated, release | 116 pass / 2 skip | no | `n-run1.log` |
| n-run2 | isolated, release | 13 pass / 4 fail / 100 not run | **yes** `subscriber_traits.rs:112` | `n-run2.log` |
| n-run3 | isolated, release | 116 pass / 2 skip | no | `n-run3.log` |
| n-dbginfo-run1..6 | isolated, release + `leptos_debuginfo` + line tables | 116 pass ×6 | no | `n-dbginfo-run{1..6}.log` |
| n-e2e-path-run1 | `cargo leptos end-to-end --release`, own port 3961 + own DB | 88 pass / 1 fail / 28 not run | **yes** `traits.rs:394` | `n-e2e-path-run1.log` |
| n-e2e-path-run2 | same | 110 pass / 1 fail / 5 not run | **yes** `subscriber_traits.rs:112` | `n-e2e-path-run2.log` |
| n-e2e-path-run3 | same | 116 pass | no | `n-e2e-path-run3.log` |
| stress, abort-requests | debuginfo binary + `n-stress.sh.txt` (48 curl/round, `--max-time` 3–80 ms, authed + anon, `/ /admin /onboarding /login`) | panic in 8/8 server lifetimes, rounds 4–47; server **dead** (abort) | **yes** `context.rs:213` | `n-stress-abort-debuginfo-backtrace.log` |
| stress, full requests | debuginfo binary + `n-stress-full.sh.txt` | panic round 25 | **yes** `context.rs:213` | `n-stress-noabort.log` |
| stress, **panic=unwind** | `CARGO_PROFILE_RELEASE_PANIC=unwind` binary, 80 abort rounds | 4 panics, all `context.rs:213`; server **alive**; `/ /admin /login` → 200 | logged, not fatal | `n-stress-unwind.log` |

Totals: isolated harness 1/12 runs crash (n-runs 1/3, dbginfo 0/6, B 0/3). cargo-leptos path 4/5 (A 2/2, n-e2e-path 2/3). The stress probe is the fast deterministic repro: one panic within seconds.

## Root cause

A library defect in `leptos_i18n` 0.6.1/0.6.2: the server-side isomorphic effect reads an arena signal from a detached task without a disposal guard. The project's release profile turns that recoverable task panic into process death. Project code has no other `.get()` inside spawned or async tasks (`grep` of `src/`: only `toast.rs` timers and the `login.rs` interval, both client-gated).

Deferred-item wording "(intermittent `tower_http` 500s)" is a misattribution. The panic runs in a detached task and cannot produce a 500. The `tower_http::trace::on_failure … 500` lines in the A logs (e.g. `f-e2e-run2.log:56`, test "1.1 — used invite code is rejected") are expected server-fn error responses.

## Fix proposal

Minimal, project-side, verified (stress: 4 panics absorbed, server serves 200s). Unwind only on the server; WASM keeps abort for size:
```diff
--- a/Cargo.toml
+++ b/Cargo.toml
@@ [profile.release]
 strip = true
 lto = "thin"
-panic = "abort"
+# WHY: unwind on the server. leptos_i18n's SSR isomorphic effect can run after the
+# request Owner is disposed and panic in a detached tokio task (context.rs:213);
+# under abort that kills the whole server. Unwind confines it to the task.
+panic = "unwind"

 [profile.wasm-release]
 inherits = "release"
 opt-level = 'z'
 lto = true
 codegen-units = 1
+panic = "abort"
```
Gates for the implementer: wasm brotli size unchanged (≈471 KB; `wasm-release` keeps abort), standard gates, and E2E via the **cargo-leptos path** ×3 (the isolated harness rarely triggers the bug). Optionally add a CI-able guard: run the stress script against the release binary and assert the server is alive afterwards.

Root fix (upstream, file an issue/PR on leptos_i18n; do not `[patch.crates-io]` locally without running that crate's tests, per conventions):
```diff
--- leptos_i18n/src/context.rs (init_context_inner)
     Effect::new_isomorphic(move |_| {
-        let new_lang = locale_signal.get();
+        let Some(new_lang) = locale_signal.try_get() else { return };
         set_lang_cookie.set(Some(new_lang));
     });
```
(Same change for the `unified_contexts` branch.) A secondary fix also belongs in reactive_graph: `new_isomorphic` should skip its first run once its owner is disposed.

## Production exposure

`Dockerfile:14` runs `cargo leptos build --release`, which uses `[profile.release]` with `panic = "abort"`. That is the same variant that crashed here. No harness is involved in prod, so the trigger is concurrency plus fast responses, the same condition the full-request stress hit in 3.6 s. Any concurrent burst can kill the prod server. Exposure: **live, high**, until the profile fix ships.

Cleanup: worktree `/home/user/wt-panic` removed (target dirs deleted first); DBs `samete_panic*`, `samete_pdbg*`, `samete_pe2e*`, `samete_pstress` dropped; no repo file modified; no commits.
