# l-toolchain — SSR/WASM build breakage root cause

Bound: stop-yapping, dry, first-principles, kiss; systematic-debugging. Investigation only, nothing committed.

## Root cause (one line)
Source growth, not toolchain drift. Commit `ee816fd` (2026-07-09, "feat(auth): page-frame/auth-card wrap, link back-buttons, OTP resend affordance") deepened the login view; later admin view growth did the same to the hydrate lib. rustc's default `recursion_limit` (128) is now exceeded by layout computation of Leptos `into_any` async blocks ("query depth increased by 130"). Both crate roots need `#![recursion_limit = "256"]`.

Second defect, bigger: CI has been **false green since 2026-07-10**. `cargo leptos end-to-end` exits 0 when the cargo build fails (EXIT=0 in all four f-e2e logs: rustc 1.97.0, 1.97.1 + cargo-leptos 0.3.7, 1.97.1 + 0.3.11, 1.99.0). CI E2E job then skips Playwright and reports success.

## Reproduction table (all with `SQLX_OFFLINE=true`, `CARGO_TARGET_DIR=/home/user/same-te-mail-club/target`, HEAD bfc2c88)

| Command | Toolchain | Result | Names |
|---|---|---|---|
| `cargo build --no-default-features --features ssr` (debug, bin) | 1.99.0 | FAIL, depth +130 | `into_any::resolve` async block over `pages::login::__component_login_step_router` / `__component_name_collection_form` closures (Div > Div > Img ... auth-card tree) |
| same, lib part | 1.99.0 | OK (lib compiles; only bin fails) | |
| `cargo build --release ... ssr` (bin) | 1.97.0 (CI), 1.97.1, 1.99.0 (readiness f-e2e logs) | FAIL, same error | same |
| `cargo build --lib --target wasm32-unknown-unknown --no-default-features --features hydrate` (debug) | 1.99.0 | FAIL, depth +130 | `into_any::hydrate_async` over `src/admin/page.rs:1841-1872` (invite-code `<ul>` / `<li>` list, `leptos_i18n macros.rs:100`) |
| wasm-release lib (cargo leptos) | 1.97.0, 1.97.1, 1.99.0 | 1.97.0: OK (07-10..07-12 CI), 1.97.1 and 1.99.0: FAIL (07-25 CI, readiness) | same as above |
| `cargo clippy --features ssr` / `cargo test --features ssr` | 1.99.0 | PASS (readiness logs; test-ssr 75 passed) | |
| `cargo build ... ssr` + `#![recursion_limit = "256"]` in `src/main.rs` only | 1.99.0 | PASS (also with 192) | |
| `cargo build --lib wasm hydrate` + `#![recursion_limit = "256"]` in `src/lib.rs` | 1.99.0 | PASS | |

Logs: `readiness/logs/l-debug-bin-1.99.0.log`, `l-debug-bin-1.99.0-rl256.log`, `-rl256-mainonly.log`, `-rl192.log`, `l-wasm-lib-debug-1.99.0.log`, `-rl256.log`, `l-bisect-<sha>-stable.log`.

## Reconciling the contradiction (clippy/test pass, build fails)
The overflow is in **layout of async-block types during codegen / monomorphization**. `cargo check` and `cargo clippy` never run codegen. `cargo test` builds the lib and a test-harness bin; the generic `into_any::resolve` / `hydrate_async` are only instantiated where `App` is reachable from a live `main` (bin: `generate_route_list(App)`/`leptos_routes`; wasm lib: `hydrate()` export). Test harness replaces `main`, so nothing instantiates them. Only a real `cargo build` of the bin or the wasm lib trips it. CI "Check" job never does either; the E2E job does, but the failure is swallowed (see below).

## Boundary
- Toolchain at last real-looking CI: runs 2026-07-10 through 2026-07-12 used rustc **1.97.0** (runner image carried 1.96.1, `stable` updated to 1.97.0). Run 2026-07-25 (30162383205) used **1.97.1 (2026-07-14)**. CI log header: `rustc 1.97.1 (8bab26f4f 2026-07-14)`, cargo-leptos v0.3.7.
- The "last green CI 2026-07-25" is **not green in substance**: job log shows `error: queries overflow the depth limit!` for lib (14:56:48) and bin (14:59:18), then `Post job cleanup`, conclusion success. Zero Playwright tests ran. Same pattern in runs 29118276831 (07-10), 29132991848 (07-11), 29151064861 (07-11), 29177651904 and 29189498352 (07-12). Logs before 07-10 are expired (HTTP 410); the 07-04 run (28719895709) took 5m53s in the E2E step, consistent with a real run.
- Source bisect on a **fixed** toolchain (1.99.0, HEAD Cargo files, old `src/`+`locales/`+`.sqlx/`): `f3291e5` (07-04) OK; `9c15ed1` (parent of ee816fd) OK; `ee816fd` FAIL; `07d2fc0` FAIL; `5da012b` FAIL; `443efdd` FAIL. Trigger commit = `ee816fd`.
- Not tested: pre-1.97 toolchain on current source. Cost: full dep rebuild (~8 GB) on a disk with ~4-9 GB free; I hit ENOSPC once (see Notes). Web reports (rust-lang/rust issues for "queries overflow the depth limit" on 1.94-1.99, Leptos/surrealdb/lance/codex reports; rust 1.97 added async wrapper layers) say newer rustc raises async layout depth, so toolchain raises the baseline, but the 128 -> over-limit crossing here is caused by our view depth. Whether 1.96.1 would pass HEAD is open; irrelevant to the fix.
- 1.97.1 fails (readiness f-e2e logs, CI). 1.99.0 fails (local repro).

## Which crate root
Error text says crate `samete` for both targets, but they are separate compilations: bin (`src/main.rs`) and lib (`src/lib.rs`). Attribute is per crate root, does not propagate. Bin overflow site is the login tree (instantiated in the bin). Lib overflow site is the admin invite list (instantiated by wasm `hydrate`). Both roots needed. Idiomatic: yes; the compiler's own help text prescribes it and it is the standard fix reported by other Leptos projects. Restructuring (splitting `view!` into smaller components) lowers depth but needs per-view work and regresses as views grow; pinning the toolchain hides the cause and the attribute is still needed on 1.97+ (CI already on 1.97.x).

## Recommended fix (exact diff)
`src/main.rs` (line 1, before the `#[cfg(feature = "ssr")]`):
```diff
+// WHY: rustc computes layout of Leptos `into_any` async blocks during codegen; the login
+// view tree is deep enough that the default limit (128) overflows ("queries overflow the
+// depth limit!", depth +130). Limit is per crate root; the lib needs its own (lib.rs).
+#![recursion_limit = "256"]
 #[cfg(feature = "ssr")]
 #[tokio::main]
```
`src/lib.rs` (top; keep before the `#![allow]`, inner attributes must precede items):
```diff
+// WHY: wasm `hydrate` instantiates `into_any::hydrate_async` over deep admin views (invite-code
+// list); layout depth exceeds the default limit (128). See main.rs for the bin counterpart.
+#![recursion_limit = "256"]
 // Leptos components return views used in macros, not directly by callers
 #![allow(clippy::must_use_candidate)]
```
256 chosen over 192 (192 also passes): rustc's own suggested value, headroom for growth.

## CI guard (decision: do NOT pin the toolchain)
Reason: unpinned `stable` is what surfaced this; a pin would have hidden it until the next bump. The real defect is that the failure was invisible. Two changes in `.github/workflows/ci.yml`, "Check" job, after clippy steps (both trigger codegen, both reproduced the failure locally in 1-5 min):
```yaml
      - name: cargo build (SSR bin, codegen)
        run: cargo build --no-default-features --features ssr
      - name: cargo build (hydrate lib, codegen)
        run: cargo build --lib --target wasm32-unknown-unknown --no-default-features --features hydrate
```
And make the E2E step fail when the build fails: `cargo leptos end-to-end` returns 0 on build errors, so add to the E2E job after "Run E2E tests (release)": `test -f end2end/test-results/.last-run.json || { echo "Playwright never ran"; exit 1; }` (or assert `end2end/results.json` exists; path per playwright.config.ts reporter). Either guard alone closes the hole; the build steps are the minimal one and also catch it in the cheaper Check job.

## Notes
- Disk filled (ENOSPC) twice during bisect (shared target, concurrent agent builds). Early bisect lines for 9c15ed1/5da012b/443efdd were invalid (link bus error / rmeta write failure) and were re-run. I removed only `samete`/`rmeta`/`rustc*` artifacts and `debug/incremental/samete-*` that I created today; nothing else deleted.
- Repo was shallow; I ran `git fetch --unshallow origin` (history now 472 commits; no working-tree change). Scratch worktree removed.
- No tracked file modified.
