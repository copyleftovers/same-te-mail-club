# Spec Review: worktree-agent-ae294272a7d5f6210

Verdict: FAIL
Worktree: /home/user/same-te-mail-club/.claude/worktrees/agent-ae294272a7d5f6210
Branch: worktree-agent-ae294272a7d5f6210
HEAD SHA: bb69736fe3fda3111eae0738f5e776777719d436
Reviewed at: 2026-10-07T21:38Z
Diff scope: git diff 5ef6072..bb69736 (U2 commit 5772ea3 + merge cdebed5 + fixups 99a1433, bb69736)
Files reviewed:
- src/config.rs
- src/main.rs
- src/sms.rs
- src/auth.rs
- src/pages/login.rs (request_otp)
- src/pages/home.rs
- README.md
- .env.example
- src/db.rs, .sqlx/ (net diff vs 5ef6072: empty — merge resolution left U3's db.rs and the cache identical to base)
- end2end/tests/fixtures/{mail_club_page.ts,cached-context.ts}, end2end/tests/mail_club.spec.ts, end2end/tests/visual-audit.spec.ts (failure analysis only)

## Findings

FAIL -- Issues found:

Misinterpreted / regression (BLOCKER — E2E gate fails; root-caused, U2-caused):
- src/pages/home.rs:156-160 (`fn test_mode()`) + call sites home.rs:188 (`resolve_preparation_state`), :406 (`enroll_in_season`), :529 (`confirm_ready`) — `use_context::<Config>()` is called AFTER several `.await`s (require_auth, sqlx fetches). In the SSR Resource path the reactive owner is not reliably current after an await, so the lookup intermittently returns None and the resource serializes `Err(ServerError("no config in context"))`. Captured in the failing page's DOM (instrumented run u2b): `__RESOLVED_RESOURCES[3] = "{\"Err\":{\"ServerError\":\"no config in context\"}}"` alongside `__RESOLVED_RESOURCES[5] = {Ok:{Preparing{...}}}`. The SSR HTML shows the Preparing form, the client hydrates with the Err payload → the Suspense subtree never hydrates → `confirm-ready-button` stays `disabled` (header `logout-button` IS enabled, so the app itself hydrated; no pageerror, no pending requests at failure). The base code read an env var (no owner dependency), so this is a U2 regression. NOTE: the implementer followed PLAN U2.5 literally — the defect is in the plan's prescription; the plan text needs correcting too.
  Required fix: read the flag synchronously at the top of each server fn, before the first `.await` (e.g. `let test_mode = expect_context::<Config>().test_mode();` / or via the same helper called first), and pass `test_mode: bool` into `resolve_preparation_state` instead of looking it up inside it. No other change.
  Evidence (all on this build unless noted):
  - Implementer run 1 (/tmp/e2e_u2.log): ✘ 2.2 confirms ready; 64 passed/2 failed.
  - Reviewer isolated `full` run (scratchpad e2e1.log): ✘ #44 2.2 confirms ready + ✘ #96 (cascade); 64 passed/2 failed/exit=1.
  - Reviewer instrumented main-suite runs on U2 release binary: 3/3 FAIL, each at a confirm-ready click (#47, #46, #46).
  - Control: same instrumented suite against agent-aa08619b87efefc0d release build (5ef6072 + A1 aria-busy only, no U2), same machine/load (~15): 80 passed, 0 failed. Historical other-lane runs on related bases (a1, f1–f3, u1×3): 0 occurrences of this failure.
  - U2 tally: 5 of 6 E2E runs failed (only implementer run 2 passed). Not load noise.
  - visual-audit #96 "no active season" failure is a pure cascade: main chain stopped at 2.2 leaving an un-cancelled season, so the create-form is legitimately absent.

Merge-resolution regression (MAJOR):
- src/config.rs tests `neither_var_means_no_bootstrap`, `blank_vars_count_as_unset`, `both_vars_normalize_phone_and_trim_name` — the merge reverted integration commit 067b9bd ("test(admin): unwrap bootstrap parse results so failures show the error", reviewed+integrated on 5ef6072) back to the pre-fix `.ok()`/`Some(...)` form from the U2 lane's stale U3 base. Fix: restore 5ef6072's three assertions verbatim (`git show 5ef6072:src/config.rs` lines 117-139). Rest of conflict resolution verified correct: single `admin_bootstrap_from_vars`, `Config { database_url, sms, admin_bootstrap }`, main.rs keeps U3's `ensure_admin` block and a220687's phone-free log line, README keeps both U3 rows, db.rs and .sqlx/ net-identical to base (the stale cache file added by the merge was removed in bb69736).

Minor (non-blocking):
- src/auth.rs:123 — "Skipped entirely when `test_mode` is true." landed under the `# Errors` heading; belongs in the body above it.

Verified compliant (U2.1–U2.7, gates):
- config.rs: SmsMode enum, Config.sms, TestModeRequiresDryRun / TestModeRequiresLoopback, sms_mode_from_vars, test_mode(), check_bind_addr in one impl block, CSRF comment removed, from_env doc covers both flags/refusals — matches plan. All 9 U2 tests present and pass.
- sms.rs, auth.rs, login.rs, main.rs (bind check + warn before DB step), README table (CSRF_SECRET row removed, RUST_LOG added), .env.example — match plan.
- Grep gates: SAMETE_* only in config.rs/main.rs; no std::env::var outside config.rs except SAMETE_LOG_POOL; sms_dry_run/turbosms_*/CSRF_SECRET = 0.
- cargo fmt --check: clean. SSR clippy -D warnings: exit 0. `cargo test`: 79 passed. `cargo test --features ssr`: 106 passed, 2 ignored, 0 failed. SSR bin build: Finished (only warning = external proc-macro-error2 future-incompat note).
- Boot refusals: TestModeRequiresDryRun → count 1, exit 101; TestModeRequiresLoopback (0.0.0.0) → count 1, exit 101; positive control (dry-run+test-mode on 127.0.0.1) passes the guard, logs the WARN, then stops only at DB connect.
- wasm32 clippy: not run — the branch predates the ENV toolchain pin (rust-toolchain.toml 1.97.1 at 215f8b1); out of scope for U2. The hydrate lib itself compiled inside `cargo leptos build --release` during the E2E run.

## Reasoning

I read PLAN-blockers.md §U2 (U2.1–U2.7 + gates) and the shared-environment gates, then the full 5ef6072..bb69736 diff, checking every plan instruction against code line by line. The code matches the plan's literal text everywhere. The plan's own TDD tests pass, and so do the grep and boot-refusal gates, which I ran myself. I did not take any of these from the implementer's report.

The E2E gate is where it fails. I did not accept "flake". One failure would be a coincidence; the same confirm-ready failure in 5 of 6 runs is a defect. I copied the suite to scratch, added console, pageerror, pending-request and DOM-dump logging to the fixture (the worktree was not modified), and ran it against the U2 binary and against a no-U2 control binary built from the same base, under the same load. The control passed. U2 failed every time. The dumped DOM holds U2's own error string in the hydration payload, which ties the symptom to `home.rs::test_mode()`'s post-await `use_context`. The header hydrated, there were no JS errors and no requests were pending, so a slow WASM load is ruled out. A hydration-data mismatch in the home Suspense explains every observation.

For the merge resolution I diffed config.rs across e89d3ca (U3 lane copy), 5ef6072 (integration) and HEAD. The only lost integration change is 067b9bd's test assertions. Everything else in config.rs, main.rs, db.rs, README and .sqlx resolves correctly.

Cleanup: sibling DBs samete_u2dbg and samete_u2dbg_* were dropped; the harness tore down its own DB. The scratch suite copy was removed. The worktree has no tracked changes. I created no target/ directory; the existing one belongs to the implementer.
