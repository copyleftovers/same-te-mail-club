# PROBLEMS — Саме Те Mail Club problem register

| Field | Value |
|---|---|
| Repo | `copyleftovers/same-te-mail-club` (local: `/home/user/same-te-mail-club`) |
| Branch | `claude/loving-johnson-7l8hn5` |
| HEAD examined (code) | `1a3c8a5`. Branch tip at authoring = `5360af1`; `git diff --stat 1a3c8a5 HEAD -- src Cargo.toml Cargo.lock .github migrations scripts justfile Dockerfile end2end/tests locales` = empty → every code finding holds at tip |
| Date | 2026-10-05 (session started 2026-10-04) |
| Sources | `evidence/{a-scope,b-build,c-ci,d-ops,e-docs,f-e2e,g-authverify,i-swapverify,j-feel,l-toolchain,m-civerify,n-panic}.md`, `evidence/logs/*`, `readiness.md`, `orchestration_log/reference/deferred_items.md`, orchestrator-reported environment problems |

## How to read

- This file states problems only. Solution mapping lives in a separate tracker.
- One entry per root problem. Same root across reports = one entry citing all.
- IDs `PRB-NNN` are stable; numbering is by area block, gaps are intentional.
- Paths: `evidence/X.md` = `orchestration_log/history/2026-10-04/evidence/X.md`; `logs/Y` = `evidence/logs/Y`. Code paths are repo-relative. `T:n` = `end2end/tests/mail_club.spec.ts:n`. Spec docs: `US` = `spec/technical/User Stories.md`, `ARCH` = `spec/technical/Architecture.md`, `PS` = `spec/product/Product Spec.md`.
- `~` before a line number = approximate line given by the source report.
- Severity: BLOCKER = no safe launch / no deployable artifact / security defeat; MAJOR = user- or organizer-visible failure, or a gate that hides failures; MINOR = drift, hygiene, small friction.
- Verification: **empirical** = observed by executing; **static** = derived from source reading. "Spot-checked" = re-read by this register's author at tip.

---

## Build/CI

### PRB-001 — Release SSR binary and WASM hydrate lib do not compile
- **Severity:** BLOCKER
- **Statement:** `cargo build` of the `ssr` bin and of the `hydrate` wasm lib fails at codegen: `queries overflow the depth limit!` (query depth +130 over default `recursion_limit` 128). `cargo check`/`clippy`/`test` stay green because they never instantiate `into_any::resolve` / `hydrate_async` over `App`. Bin overflow site: login view tree (`pages::login::__component_login_step_router`, `__component_name_collection_form`). Lib overflow site: `src/admin/page.rs:1841-1872` (invite-code list). Trigger commit by source bisect on fixed toolchain: `ee816fd` (2026-07-09); parent `9c15ed1` builds.
- **Evidence:** `evidence/l-toolchain.md` (reproduction + bisect tables); `evidence/f-e2e.md` Finding 1; `evidence/m-civerify.md`; logs `logs/l-debug-bin-1.99.0.log`, `logs/l-wasm-lib-debug-1.99.0.log`, `logs/l-bisect-ee816fd-stable.log` (FAIL), `logs/l-bisect-9c15ed1-stable.log`, `logs/f-e2e-run1-rustc1.97.0-FAIL.log`, `logs/f-e2e-run1-rustc1.99.0-FAIL.log`, `logs/f-e2e-run1-rustc1.97.1-leptos0.3.7-FAIL.log`. Spot-check: `grep recursion_limit src/main.rs src/lib.rs Cargo.toml` → 0 hits.
  ```
  error: queries overflow the depth limit!
    = help: consider increasing the recursion limit by adding a `#![recursion_limit = "256"]` attribute to your crate (`samete`)
  ```
- **Reproduction:** `SQLX_OFFLINE=true cargo build --no-default-features --features ssr` and `SQLX_OFFLINE=true cargo build --lib --target wasm32-unknown-unknown --no-default-features --features hydrate` on rustc 1.97.1 or 1.99.0. Release via `cargo leptos build --release` fails identically.
- **Impact:** No deployable artifact. `Dockerfile:11` runs `cargo leptos build --release` → Coolify build fails the same way. E2E cannot run without an out-of-repo compiler override.
- **Verification:** empirical (l: local repro + bisect, rustc 1.99.0; f: rustc 1.97.0/1.97.1/1.99.0; m: CI logs rustc 1.97.0/1.97.1). Untested: whether pre-1.97 rustc builds HEAD.

### PRB-002 — CI E2E job is false-green: build failure exits 0, Playwright never runs
- **Severity:** BLOCKER
- **Statement:** `just e2e-release` → `cargo leptos end-to-end --release` exits 0 when the cargo build fails. No CI step asserts that Playwright ran, a test count, `results.json`, or the binary. Every readable CI log since 2026-07-10 shows the PRB-001 compile error, no Playwright output, and job conclusion `success`. No CI run on record provably executed Playwright (logs ≤2026-07-04 expired, HTTP 410).
- **Evidence:** `evidence/m-civerify.md` (table); `evidence/l-toolchain.md` "Boundary"; `justfile:51-52`; `.github/workflows/ci.yml` E2E job. CI runs: 30162383205 (c472bfc, 2026-07-25, rustc 1.97.1), 29189498352, 29177651904, 29151064861, 29132991848, 29118276831. f logs show `EXIT=0` on failed builds (`logs/f-e2e-run1-rustc1.97.0-FAIL.log` et al.).
  ```
  error: queries overflow the depth limit!   (lib 14:56:48, bin 14:59:18, run 30162383205)
  could not compile `samete` (bin "samete") due to 1 previous error
  -> E2E step 13 conclusion: success
  ```
- **Reproduction:** on any tree where PRB-001 holds: `just e2e-release; echo $?` → `0`. CI: `gh run view 30162383205 --log | grep -E "overflow|passed|failed"`.
- **Impact:** Every "CI green" since 2026-07-10 is unproven. Regressions in any E2E-covered flow are invisible. The same exit-0 behavior hides failures in local `just e2e*` recipes.
- **Verification:** empirical (m: CI job logs via API; l, f: local exit codes).

### PRB-003 — CI Check job never runs codegen
- **Severity:** MAJOR
- **Statement:** Check job = fmt, clippy SSR, clippy hydrate, cargo-audit, bare `cargo test`. None builds the bin or the wasm lib, so codegen-only failures (PRB-001) pass Check.
- **Evidence:** `.github/workflows/ci.yml:20-62`; `evidence/b-build.md` (all four gates green at 1a3c8a5 while build fails); `evidence/l-toolchain.md` "Reconciling the contradiction".
- **Reproduction:** run the Check job commands from `evidence/b-build.md` table at 1a3c8a5 → all exit 0; run PRB-001 command → fails.
- **Impact:** The cheap gate cannot detect an unbuildable tree; only the (false-green, PRB-002) E2E job builds.
- **Verification:** empirical (b executed the gates; l reproduced the build failure).

### PRB-004 — CI and `just test`/`just check` run bare `cargo test`; ssr-gated tests never run
- **Severity:** MAJOR
- **Statement:** Bare `cargo test` = 68 tests; `--features ssr` = 75. The 7 `auth.rs` tests (module is `#[cfg(feature = "ssr")]` in `lib.rs`) never run in CI or in `just test`/`just check`.
- **Evidence:** `evidence/b-build.md` finding 2 (68 vs 75; `logs/test-bare.log`, `logs/test-ssr.log`); `evidence/e-docs.md` #1, #2, F2; `ci.yml:52-53`.
- **Reproduction:** `SQLX_OFFLINE=true cargo test 2>&1 | grep "test result"` vs `SQLX_OFFLINE=true cargo test --features ssr 2>&1 | grep "test result"`.
- **Impact:** Auth hash/compare tests can break without any gate failing.
- **Verification:** empirical (b).

### PRB-005 — Toolchain and tool versions unpinned in CI; outcome depends on resolve date
- **Severity:** MAJOR
- **Statement:** CI uses `dtolnay/rust-toolchain@stable` (`ci.yml:21`, `:85`); no `rust-toolchain*` file in repo. `cargo binstall cargo-leptos` (`ci.yml:104`), `wasm-opt`, `sqlx-cli`, `just`, `cargo-audit` are unpinned. CI resolved rustc 1.97.0 (07-10..07-12) then 1.97.1 (07-25); the wasm-release lib built on 1.97.0 and fails on 1.97.1. binstall resolved cargo-leptos 0.3.7 in run 30162383205 and resolves 0.3.11 now.
- **Evidence:** spot-check `grep -n "toolchain\|binstall" .github/workflows/ci.yml`; `ls rust-toolchain*` → absent; `evidence/l-toolchain.md` reproduction table (wasm-release row); `evidence/f-e2e.md` "Environment deltas vs CI"; `evidence/e-docs.md` #28.
- **Reproduction:** compare `gh run view 29118276831 --log | grep "rustc 1."` with `gh run view 30162383205 --log | grep "rustc 1."`.
- **Impact:** Same commit can build or fail depending on run date; local and CI environments diverge silently.
- **Verification:** empirical (l, f, m CI logs) + static (ci.yml).

### PRB-006 — HEAD-adjacent commits carry no CI run (skip directive)
- **Severity:** MINOR
- **Statement:** `1a3c8a5`, `fc322ed`, `4f47a95` have no CI run (skip token in message; for `fc322ed` the token appeared as quoted substring). "CI green at HEAD" is by inheritance from `c472bfc`, itself false-green (PRB-002).
- **Evidence:** `evidence/c-ci.md` §3.
- **Reproduction:** `gh run list --commit 1a3c8a5`.
- **Impact:** No run attests HEAD; status claims rest on an older run.
- **Verification:** empirical (c, GitHub API).

### PRB-007 — Supply-chain audit freshness unknown
- **Severity:** MINOR
- **Statement:** `cargo audit` is a hard CI gate (exit 1 on RUSTSEC warnings; run 29188964277 failed on it). Last CI run 2026-07-25. Advisories published since are unchecked; `cargo-audit` was not run in this session.
- **Evidence:** `evidence/c-ci.md` §4 + findings; `evidence/b-build.md` finding 4.
- **Reproduction:** `cargo binstall --no-confirm cargo-audit && cargo audit`.
- **Impact:** Next push may turn red for reasons unrelated to the change; vulnerable deps may be present.
- **Verification:** unverified (not run).

### PRB-008 — Future-incompatibility warning in dependency tree
- **Severity:** MINOR
- **Statement:** `proc-macro-error2 v2.0.1` emits a future-incompat warning.
- **Evidence:** `evidence/b-build.md` finding 5; `logs/clippy-ssr.log`.
- **Reproduction:** `cargo report future-incompatibilities` after a clippy run.
- **Impact:** A future rustc may reject the dependency.
- **Verification:** empirical (b).

---

## Runtime stability

### PRB-010 — Server process dies under concurrency (leptos_i18n disposed-signal panic + `panic = "abort"`)
- **Severity:** BLOCKER
- **Statement:** `leptos_i18n-0.6.1/src/context.rs:213` reads `locale_signal.get()` inside `Effect::new_isomorphic` (`:206`). On the server that effect runs in a detached tokio task (`reactive_graph-0.2.13/src/effect/effect.rs:402-440`); the request Owner is force-disposed at stream end (`leptos_integration_utils-0.8.8/src/lib.rs:121-125`). If the response ends before the task's first poll, the read hits a disposed arena slot and panics. `Cargo.toml:98` `panic = "abort"` (release profile, inherited by prod Docker build) turns the task panic into process death. Reached from `src/app.rs:73` `provide_i18n_context()`. Not a client-disconnect bug (full-request stress also panics). `leptos_i18n` 0.6.2 has identical code. Harness-sensitive: cargo-leptos path crashed 4/5 runs, isolated harness 1/12.
- **Evidence:** `evidence/n-panic.md` (verdict, backtrace, repro table); `evidence/f-e2e.md` Failures (runs 1–2 crashed); logs `logs/n-stress-abort-debuginfo-backtrace.log` (8/8 lifetimes), `logs/n-stress-noabort.log` (round 25), `logs/n-stress-unwind.log` (unwind: 4 panics, server alive), `logs/f-e2e-run1.log:274`, `logs/f-e2e-run2.log:75`, `logs/n-run2.log`, `logs/n-e2e-path-run1.log`, `logs/n-e2e-path-run2.log`. Spot-check: `Cargo.toml:98` = `panic = "abort"`.
  ```
  panicked at .../reactive_graph-0.2.13/src/traits.rs:394:39:
  At .../leptos_i18n-0.6.1/src/context.rs:213:38, you tried to access a reactive value which was
  defined at .../leptos_i18n-0.6.1/src/context.rs:307:19, but it has already been disposed.
  ```
- **Reproduction:** build a release binary (requires getting past PRB-001; n used an uncommitted `#![recursion_limit = "256"]`), optionally with `RUSTFLAGS='--cfg leptos_debuginfo'`, run it, then `logs/n-stress.sh.txt` (48 curls/round, `--max-time` 3–80 ms, `/ /admin /onboarding /login`) → panic within seconds, process exits. E2E: `cargo leptos end-to-end --release` → crash in most runs, cascading `ERR_CONNECTION_REFUSED`.
- **Impact:** Any concurrent burst can kill production. E2E suite cannot be 3×-green; crash cascades produce dozens of false failures. Second panic variant `subscriber_traits.rs:112` (`logs/n-run2.log`) is attributed to the same path by inference only.
- **Verification:** empirical (n: debuginfo backtrace, stress, unwind contrast; f: two crashed E2E runs).

---

## Security/Auth

### PRB-020 — Registration bypasses OTP via forgeable `pending_phone` cookie
- **Severity:** BLOCKER
- **Statement:** `verify_otp_code` sets `pending_phone` to the raw normalized phone, unsigned, unbound to server state (`src/pages/login.rs:214`). `validate_invite_code` checks only cookie presence (`login.rs:323`); `register_with_code` takes the phone from the cookie (`login.rs:391-396`, parser `login.rs:487-491`). No server-side record of OTP verification exists. Any client sending `Cookie: pending_phone=<any phone>` plus one valid invite code obtains a user row and session for a phone it does not own. Existing-account takeover is blocked by `users.phone UNIQUE`.
- **Evidence:** `evidence/f-e2e.md` "Auth bypass empirical check" (live: 200, `set-cookie: session=…`, `location: /onboarding`, invite row `used`, session row for `+380999000222`); `evidence/g-authverify.md` 1b, 1c; `evidence/d-ops.md` F1. Spot-checked `login.rs:214,323,391,491` at tip.
  ```
  curl -sS -i -X POST "http://127.0.0.1:<port>/api/register_with_code4575314452317009345" \
    -H "Cookie: pending_phone=+380999000222" --data-urlencode "code=bypass-test" --data-urlencode "name=Bypass Victim"
  HTTP/1.1 200 OK
  set-cookie: session=uGQPcaL93lQGeIc7RjE9EXvLgefcS9ra6JseD8dqxtU; HttpOnly; Secure; SameSite=Strict; Max-Age=7776000; Path=/
  ```
- **Reproduction:** server with `SAMETE_TEST_MODE` unset; insert unused invite code via `psql`; resolve the hashed server-fn path from the rendered form; send the curl above cold (no `/login`, no `request_otp`, no `verify_otp_code`).
- **Impact:** OTP authentication defeated for registration. Anyone with one invite code can squat any unclaimed phone; the real owner later logs into an account with attacker-chosen name/address.
- **Verification:** empirical (f, live against running server); static (g, d).

### PRB-021 — Invite codes: low entropy, unthrottled redemption, distinct-error oracle, no expiry
- **Severity:** MAJOR
- **Statement:** Codes = 2 distinct words from a 200-word list → 39,800 codes (~15.3 bits), format `a-b` (`src/invite_codes.rs:262`). `validate_invite_code`/`register_with_code` have no attempt counter, IP or phone limit. Errors distinguish invalid/used/revoked (`login.rs:~280-300`), leaking code status. No `expires_at` column (`migrations/20260314000003_*`). Combined with PRB-020, enumeration needs no phone.
- **Evidence:** `evidence/g-authverify.md` Q2; `evidence/d-ops.md` F1; `evidence/a-scope.md` row 1.1 ("Codes never expire", retry unlimited by US design).
- **Reproduction:** with a forged `pending_phone` cookie, loop `validate_invite_code` over the word-pair space; observe no throttling and distinct error strings.
- **Impact:** Outstanding unused codes are guessable at rate N/39,800 per request; code status leaks.
- **Verification:** static (g, d).

### PRB-022 — `SAMETE_TEST_MODE=true` disables OTP and rate limits with no guard
- **Severity:** BLOCKER
- **Statement:** When `SAMETE_TEST_MODE` = `"true"`: every OTP is `000000` (`src/auth.rs:77`), OTP rate limit skipped (`auth.rs:121`), deadline gates skipped (`src/pages/home.rs:180,398,521`). No boot-time refusal, warning, or log line. `.env.example:2` exports it `true`. README env table omits it. Dockerfile sets only `LEPTOS_*`.
- **Evidence:** spot-check `grep -n SAMETE_TEST_MODE src` (5 sites above) and `.env.example`; `evidence/d-ops.md` F2 + env table; `evidence/g-authverify.md` Q3; `evidence/a-scope.md` Gap 5.
- **Reproduction:** start server with `SAMETE_TEST_MODE=true`; `request_otp` for the admin phone; `verify_otp_code` with `000000` → admin session.
- **Impact:** One env misconfiguration in Coolify = anyone logs in as any existing user, including admin.
- **Verification:** static (d, g); behaviour of test mode exercised empirically by every E2E run (f, n).

### PRB-023 — SMS dry-run silently disables SMS and logs OTP codes
- **Severity:** MAJOR
- **Statement:** `SAMETE_SMS_DRY_RUN=true` removes the TurboSMS credential requirement, sends nothing, and logs full phone + message body (including OTP code) at `info` (`src/sms.rs:41-45`). Boot logs nothing about the mode. README omits the variable; `.env.example:3` sets it `true`.
- **Evidence:** `evidence/d-ops.md` F8 + env table; `evidence/g-authverify.md` Q3.
- **Reproduction:** run with `SAMETE_SMS_DRY_RUN=true`, `RUST_LOG=samete=info`; call `request_otp`; grep server log for the 6-digit code.
- **Impact:** Production misconfig = no participant ever receives SMS (including OTP), and OTP codes sit in logs.
- **Verification:** static (d, g).

### PRB-024 — SMS pumping: unauthenticated OTP requests, phone-keyed limits only, declared lockout and global cap missing
- **Severity:** MAJOR
- **Statement:** `request_otp` (public, `login.rs:39-100`) sends a real SMS to any number. Limits are per phone only (1/60s, 5/h, `auth.rs:134-170`). No IP dimension (`grep ConnectInfo|x-forwarded|remote_addr src` = 0). ARCH:169 (10 failed verifies/phone/hour → 1h lock) and ARCH:171 (global ~50 SMS/h circuit breaker) are not implemented. OTP is also sent to deactivated phones.
- **Evidence:** `evidence/d-ops.md` F3 + deferred re-verify table; `evidence/a-scope.md` row "1.2 (ARCH) Lock after 10 failed verifies…" = MISS; `deferred_items.md` "IP-based OTP rate-limiting absent" (first flagged 2026-06-22). Spot-check: ARCH:167-171.
- **Reproduction:** script `request_otp` across many distinct numbers; each gets one SMS; no aggregate stop.
- **Impact:** Unbounded TurboSMS spend (~$0.025/SMS) and SMS-bombing of third parties.
- **Verification:** static (d, a); deferred item re-verified open by d.

### PRB-025 — `request_otp` is an account-existence oracle
- **Severity:** MAJOR
- **Statement:** `request_otp` returns `NewAccount` vs `AccountExists` to the requester (`login.rs:63-70`); doc comment claims the outcome is visible only to the phone holder. Rate-limited path masks as `AccountExists`, success path does not.
- **Evidence:** `evidence/d-ops.md` F4; `evidence/g-authverify.md` "Other trust edges".
- **Reproduction:** call `request_otp` for a member and a non-member phone; compare responses.
- **Impact:** Club membership of any phone number is disclosed to anyone.
- **Verification:** static (d, g).

### PRB-026 — OTP/session hygiene gaps
- **Severity:** MINOR
- **Statement:** (a) OTP attempt counter is read-then-write, non-atomic: parallel requests can exceed 3 attempts per code. (b) OTP stored as unsalted SHA-256 of 6 digits (10^6 space; DB leak = instant recovery). (c) Expired session/OTP rows deleted only when accessed; no sweep.
- **Evidence:** `evidence/g-authverify.md` Q3; `evidence/d-ops.md` security table + F10; `src/auth.rs:~81,~100,~190`.
- **Reproduction:** (a) fire N concurrent `verify_otp_code` with wrong codes for one phone; count accepted attempts.
- **Impact:** Small brute-force margin above the declared 3/code; table growth.
- **Verification:** static (g, d).

### PRB-027 — `register_with_code` discards DB results and hides failures
- **Severity:** MINOR
- **Statement:** `UPDATE` of the invite code and `commit` results are discarded with `let _ =` (`login.rs:~437-445`); a failed commit still proceeds to mint a session. Failure paths redirect with no error message (`login.rs:424-439`).
- **Evidence:** `evidence/g-authverify.md` Q2 last bullet; `evidence/a-scope.md` row 1.1 "Invalid/used/revoked" note.
- **Reproduction:** static only (requires DB fault injection at commit).
- **Impact:** Silent registration failure; user sees `/login` with no explanation.
- **Verification:** static (g, a).

### PRB-028 — Admin can deactivate self or another admin
- **Severity:** MINOR
- **Statement:** `deactivate_participant` (`src/admin/participants.rs:76-101`) has no guard against the caller's own account or admin accounts.
- **Evidence:** `evidence/a-scope.md` row 6.1, Gap 8.
- **Reproduction:** as sole admin, deactivate own row → sessions deleted, no admin remains (compounds PRB-070).
- **Impact:** Organizer lockout recoverable only via SQL.
- **Verification:** static (a).

---

## Data/Domain logic

### PRB-030 — Assignment swap (Story 3.3) can never succeed; structurally unsafe
- **Severity:** BLOCKER
- **Statement:** `swap_assignment` (`src/admin/assignments.rs:318-416`) issues two independent `UPDATE … SET recipient_id` on the pool (no transaction), then validates. (1) The first UPDATE violates non-deferrable `UNIQUE (season_id, recipient_id)` (`migrations/20260314000002_create_tables.sql:66`) → every swap with `sender_a ≠ sender_b` errors. (2) The recipient-exchange semantic always splits a single cycle into two (or a self-loop for adjacent senders), so validation would fail even without the constraint. (3) Validation runs after writes with no rollback. (4) `validate_swap_topology` wraps all rows as one cohort (`assignments.rs:187-217`), so any multi-cohort season (>15, `split_cohorts`) fails even a no-op. (5) `sender_a == sender_b` is accepted (UI lists all senders in both selects) — the only "successful" call is a no-op. E2E T:649 passes regardless.
- **Evidence:** `evidence/i-swapverify.md` (Q table, math, 3/4/5-node cases, unwritten failing unit test); `evidence/a-scope.md` Gap 1 (earlier reading "persists corruption" superseded by i: UNIQUE fails first). Spot-check: UPDATEs before `validate_swap_topology` call; `UNIQUE (season_id, recipient_id)` at line 66; `grep -ri DEFERRABLE migrations` = empty (i).
- **Reproduction:** E2E chain to Assignment phase with 3 participants, generate, select two different senders, submit swap → `action-error` with unique-violation text (not asserted by T:649). Math: cycle a→b→c→d→a, exchange recipients of a and c → a→d→a, b→c→b.
- **Impact:** Organizer cannot adjust assignments at all. If the constraint were ever relaxed, a split graph would persist and be released (PRB-031).
- **Verification:** static (i, a). Not executed.

### PRB-031 — Phase advance does not validate assignments
- **Severity:** MAJOR
- **Statement:** `advance_season` (`src/admin/season.rs:144-176`) + `Phase` transitions (`src/types.rs:28`) allow Assignment→Delivery with zero assignments and never call `validate_cycles`. The guard exists only as a disabled UI button (`src/admin/page.rs:537`). ARCH:518 "advance disabled until confirm deadline passes" is not implemented.
- **Evidence:** `evidence/a-scope.md` rows 4.7 (both), Gap 1; `evidence/i-swapverify.md` Q3 (`grep -rn validate_cycles src` → generate + swap only).
- **Reproduction:** POST `advance_season` server fn directly from Assignment phase with no `assignments` rows → succeeds.
- **Impact:** Participants can be released into Delivery with no or invalid assignments.
- **Verification:** static (a, i).

### PRB-032 — Assignment generation deletes existing rows before guard, outside a transaction
- **Severity:** MINOR
- **Statement:** `generate_assignments_action` DELETEs prior assignments (`assignments.rs:259`) before the N≥3 guard (`:279`), no transaction.
- **Evidence:** `evidence/a-scope.md` row 3.1 "Generation guard", Gap 8.
- **Reproduction:** generate, deactivate/unconfirm until <3 confirmed, regenerate → error, previous assignments gone.
- **Impact:** A failed regenerate wipes valid assignments.
- **Verification:** static (a).

### PRB-033 — Deactivated users stay in cohort pool and nudge targets
- **Severity:** MAJOR
- **Statement:** No `status` filter in the confirmed-participant pool query (`assignments.rs:273-276`) or in confirm/receipt nudge queries (`src/admin/sms.rs`). Season-open SMS filters `status='active'` (`sms.rs:182`); the others do not. A participant deactivated after confirming is placed in a cycle but cannot log in to see their recipient. (a rates MINOR; impact below sets MAJOR.)
- **Evidence:** `evidence/a-scope.md` Gap 8; row 6.1 "Excluded from season-open SMS".
- **Reproduction:** enroll + confirm user X, deactivate X, generate → X appears in `cycle-visualization`.
- **Impact:** Broken mail chain: X's recipient gets nothing; X gets SMS nudges after removal.
- **Verification:** static (a).

### PRB-034 — Assignment SMS sendable before assignments are released
- **Severity:** MINOR
- **Statement:** Server allows `send_assignment_sms` in the `assignment` phase (`sms.rs:256`); only the UI hides the button.
- **Evidence:** `evidence/a-scope.md` row 2.3 "SMS nudge when available".
- **Reproduction:** call the server fn directly during Assignment phase.
- **Impact:** Participants told to check an assignment they cannot yet see.
- **Verification:** static (a).

### PRB-035 — Cancel offered before launch (contradicts Story 4.3)
- **Severity:** MINOR
- **Statement:** Cancel branch renders for `!is_terminal`, ignoring `launched` (`page.rs:733`); US 4.3: "Cancel is only available in the UI after launch".
- **Evidence:** `evidence/a-scope.md` row 4.3 (CONTR), Gap 8.
- **Reproduction:** create a season, do not launch, open `/admin` → cancel control present.
- **Impact:** Spec/code disagreement; low user harm.
- **Verification:** static (a).

---

## Product/UX

### PRB-040 — Home state ignores participation: dead CTA and false promises to non-participants
- **Severity:** MAJOR
- **Statement:** (a) Preparation: confirm-ready CTA shown to any non-confirmed user, enrolled or not (`src/pages/home.rs:162-186`); click UPDATEs 0 rows and returns Ok (`home.rs:530-542`); doc comment at `:496` says it errors. (b) Assignment/Delivery: users with no outgoing assignment see "Розподіл · В процесі · За кілька хвилин…" for the whole window (`home.rs:222-225,336`); Complete tells them "Дякуємо, що був(ла) з нами" (`home.rs:317,339`; `locales/uk.json:29,43`).
- **Evidence:** `evidence/a-scope.md` Gap 4, row 2.2; `evidence/j-feel.md` P3.
- **Reproduction:** register a user who does not enroll; walk the season through Preparation → Delivery; observe states as that user.
- **Impact:** Non-enrolled users believe they are in the graph; non-participants wait for mail that never comes.
- **Verification:** static (a, j).

### PRB-041 — Nova Poshta branch cannot be changed after onboarding
- **Severity:** MAJOR
- **Statement:** US 1.3 AC3 and 2.1 AC2 promise branch update during enrollment or in settings. Every user is onboarded, so the enrollment form always takes the saved-address branch with no inputs (`home.rs:820-849`); no settings route (`src/app.rs:100-145`); onboarding guard redirects onboarded users away (`app.rs:283`).
- **Evidence:** `evidence/a-scope.md` rows 1.3, 2.1 (MISS), Gap 2.
- **Reproduction:** onboarded user opens enrollment → no address inputs; `/onboarding` redirects to `/`.
- **Impact:** A wrong or moved branch sends mail to the wrong Nova Poshta office; correction requires organizer SQL.
- **Verification:** static (a).

### PRB-042 — Organizer sees counts, not people; receipt notes never displayed
- **Severity:** MAJOR
- **Statement:** Admin state exposes enrolled/confirmed/not-received/no-response as integers only (`src/admin/state.rs:33-43`, `page.rs:555-645`; `uk.json:84` "Не отримано: N"). `receipt_note` is written (`home.rs:621`) and read nowhere. Participant table has no per-season status (`page.rs:2100-2103`). PS Failure Protocol needs non-receiver identity; ARCH:520 promised per-participant receipt status.
- **Evidence:** `evidence/a-scope.md` row 2.4 "not received", Gap 3, row "PS Failure protocol"; `evidence/j-feel.md` O1.
- **Reproduction:** participant reports "not received" with a note → `/admin` shows count +1; note and name appear nowhere.
- **Impact:** Organizer must query the DB to nudge individuals or run the forwarding protocol.
- **Verification:** static (a, j).

### PRB-043 — Recipient details vanish when a participant confirms their own receipt
- **Severity:** MAJOR
- **Statement:** `ReceiptConfirmed` replaces `Assigned` and carries no recipient (`home.rs:241-244`; render `home.rs:1094-1121`). Sending and receiving are independent.
- **Evidence:** `evidence/j-feel.md` P1.
- **Reproduction:** in Delivery, as a participant who has not shipped yet, confirm receipt → recipient name/phone/branch gone.
- **Impact:** Participant who receives before shipping loses the address needed to ship.
- **Verification:** static (j).

### PRB-044 — Receipt form shown on day 0; "not received" is irreversible
- **Severity:** MAJOR
- **Statement:** Receipt form renders together with the assignment (`home.rs:1086`), not ~5 days after (US 2.4, PS:43). "Не отримав" is final: UPDATE only where `receipt_status = 'no_response'` (`home.rs:592-597`). No delivery-start timestamp exists (seasons has no phase timestamps).
- **Evidence:** `evidence/j-feel.md` P2; `evidence/a-scope.md` row 2.4 "Prompt 5 days after" (PART).
- **Reproduction:** open home on Delivery day 0, tap "Не отримав" → permanent `not_received`.
- **Impact:** Early or accidental taps create permanent false failure signals for the forwarding protocol (and trigger PRB-043).
- **Verification:** static (j, a).

### PRB-045 — No meetup information anywhere
- **Severity:** MAJOR
- **Statement:** The meetup is the product's payoff (PS, Introduction). US 2.1 (`US:135`) promises an "expected meetup window". `grep -rni meetup src migrations` = 0; seasons table has no meetup column (`migrations/20260314000002_create_tables.sql:34-42`). Enrollment screen also lacks the creation deadline.
- **Evidence:** `evidence/j-feel.md` P10; `evidence/a-scope.md` row 2.1 timeline (PART), Gap 7.
- **Reproduction:** walk a full season as participant; no date/place shown; Complete says "До наступного разу".
- **Impact:** Participants depend on side channels for the core event.
- **Verification:** static (j, a).

### PRB-046 — SMS messages contain no link to the app
- **Severity:** MAJOR
- **Statement:** SMS bodies are bare locale keys ("Заходь в додаток") with no URL (`uk.json:98,99`; `sms.rs:191,283`); no site-URL config in `src/config.rs`.
- **Evidence:** `evidence/j-feel.md` P11.
- **Reproduction:** trigger any SMS with dry-run on; inspect logged body.
- **Impact:** Monthly-cadence web app; participants must remember the URL to act on every notification.
- **Verification:** static (j).

### PRB-047 — Forwarding protocol unsupported by the cycle view
- **Severity:** MAJOR
- **Statement:** `AssignmentLink {sender_id, sender_name, recipient_name}` (`assignments.rs:25-29`) carries no receipt status; the PS non-compliance / contiguous-failure forwarding rule must be traced mentally.
- **Evidence:** `evidence/j-feel.md` O2.
- **Reproduction:** Delivery with ≥1 not-received report → cycle viz identical to no-failure state.
- **Impact:** Organizer under time pressure computes forwarding pairs by hand from DB data.
- **Verification:** static (j).

### PRB-048 — No organizer notification on "not received"
- **Severity:** MAJOR
- **Statement:** US 2.4 (`US:187`): "not received" triggers organizer notification. Only a polled count exists (`page.rs:637-645`).
- **Evidence:** `evidence/j-feel.md` O3; `evidence/a-scope.md` row 2.4 (PART).
- **Reproduction:** report not-received; no SMS/alert to admin.
- **Impact:** Failures surface only when the organizer happens to open `/admin`.
- **Verification:** static (j, a).

### PRB-049 — Season dates and theme cannot be edited after creation
- **Severity:** MAJOR
- **Statement:** Season server fns are create/launch/advance/cancel only (`season.rs:20,106,144,184`); only UPDATEs are launch/phase/cancel (`season.rs:114,167,214`). Extending a deadline requires cancel+recreate (strands enrollments) or SQL.
- **Evidence:** `evidence/j-feel.md` O6.
- **Reproduction:** create a season with a wrong date; no edit control or server fn.
- **Impact:** Routine deadline extensions require destructive workarounds.
- **Verification:** static (j).

### PRB-050 — Admin submit buttons emit literal `attr:aria-busy` attribute
- **Severity:** MAJOR
- **Statement:** 13 native `<button>` elements in `src/admin/page.rs` use `attr:aria-busy=` (e.g. `:497,675,705,753,865,906,955,992,1102,1647`). While pending, the DOM gets an attribute literally named `attr:aria-busy`; idle state emits nothing. Assistive tech never sees `aria-busy`. Same Leptos hazard as the 2026-07-04 `attr:aria-invalid` incident.
- **Evidence:** `evidence/f-e2e.md` "attr: leak check" (`getAttributeNames()` → `"attr:aria-busy=true"` on `create-season-button` mid-submit); `evidence/e-docs.md` #44, F1.
  ```
  ["type=submit","data-testid=create-season-button","class=btn","attr:aria-busy=true","disabled="]
  ```
- **Reproduction:** delay the `create_season` POST, click submit, read `button.getAttributeNames()` in page.
- **Impact:** Loading state invisible to screen readers on every admin action; codebase_state claim "all submit buttons have aria-busy" false.
- **Verification:** empirical (f, hydrated DOM).

### PRB-051 — Enrollment form still offered after signup deadline
- **Severity:** MINOR
- **Statement:** `EnrollmentOpen` state has no deadline flag (`home.rs:30-35`, `107-153`); server rejects at `home.rs:399` ("Термін реєстрації вже минув").
- **Evidence:** `evidence/j-feel.md` P4; `evidence/a-scope.md` row 2.1 "Enrollment closes at sign-up deadline".
- **Reproduction:** season past signup deadline, still in Enrollment phase → form visible; submit errors.
- **Impact:** Dead-end action for the length of organizer latency.
- **Verification:** static (j, a).

### PRB-052 — Theme and content guidelines missing where needed
- **Severity:** MINOR
- **Statement:** Theme shown only in enrollment (`home.rs:799-804`); `Enrolled`/`Preparing` carry no theme (`home.rs:38-44`), contrary to PS:155 ("during enrollment and creation period"). Enrollment shows a creative brief (`home.rs:813`, `uk.json:15`) but not PS "Content Guidelines"/"Ownership & Consent" prohibitions. T:390 labelled "guidelines" asserts `season-theme`.
- **Evidence:** `evidence/j-feel.md` P5; `evidence/a-scope.md` rows 2.1 guidelines, 4.1 theme (PART).
- **Reproduction:** enroll → theme gone during creation period.
- **Impact:** Participants create without the brief; consent rules never shown.
- **Verification:** static (j, a).

### PRB-053 — No countdown to confirm deadline
- **Severity:** MINOR
- **Statement:** US 2.2 (`US:154`) asks for a countdown; Preparing shows a static date (`home.rs:945-952`).
- **Evidence:** `evidence/j-feel.md` P13; `evidence/a-scope.md` row 2.2 "Deadline visible with countdown" (PART).
- **Reproduction:** view Preparing state.
- **Impact:** Weaker deadline salience.
- **Verification:** static (j, a).

### PRB-054 — Copy defects contradict product identity or reality
- **Severity:** MINOR
- **Statement:** Pen-pal framing "знайдемо тобі пару для листування" vs PS:5 "NOT a pen-pal service" (`uk.json:11`); "Кому саме, дізнаєшся після реєстрації" false — learned after assignment (`uk.json:12`); "За кілька хвилин" for an organizer-triggered, days-long step (`uk.json:29`); advance-blocked hint references a nonexistent publish page (`uk.json:76`; action is "Згенерувати" on single `/admin`); one-line shipping guidance with no window/tier/cost (`uk.json:147`); not-received confirmation gives no next step (`uk.json:40-41`).
- **Evidence:** `evidence/j-feel.md` P6, P7, P8, P9, O7.
- **Reproduction:** read the listed keys in `locales/uk.json`.
- **Impact:** Misleading expectations for newcomers and organizer.
- **Verification:** static (j).

### PRB-055 — No orientation for newcomers between seasons
- **Severity:** MINOR
- **Statement:** First screen after onboarding with no season (`uk.json:142`, `home.rs:1132-1137`) explains nothing about the ritual; persona fears (Personas.md) unaddressed.
- **Evidence:** `evidence/j-feel.md` P12.
- **Reproduction:** onboard with no active season.
- **Impact:** Newcomer drop-off risk.
- **Verification:** static (j).

### PRB-056 — Organizer must remember separate SMS steps and nudge timing
- **Severity:** MINOR
- **Statement:** Launch sends no SMS (`season.rs:106-133`); season-open SMS is a separate button (`sms.rs:155`). Advance to Delivery likewise needs a separate assignment-SMS click (`sms.rs:238`). Confirm nudge must be clicked by hand ~1h before deadline; admin shows no time-remaining hint (`page.rs:910-930`). Manual triggering is a declared design (ARCH:37); the friction is the absence of any prompt.
- **Evidence:** `evidence/j-feel.md` O4, O5; `evidence/a-scope.md` rows 4.2, 5.4.
- **Reproduction:** launch a season; no SMS sent and no prompt to send.
- **Impact:** A forgotten click = participants never learn the season opened or who to send to.
- **Verification:** static (j, a).

### PRB-057 — Minor acceptance-criteria deviations in UI
- **Severity:** MINOR
- **Statement:** Logout not shown on `/onboarding` (US 1.4 "every authenticated page"; nav hidden on `/onboarding`, `app.rs:195`). Assignment SMS counter shows only the not-notified count M, not "N senders (M not yet notified)" (US 4.4; `page.rs:966`).
- **Evidence:** `evidence/a-scope.md` rows 1.4, 4.4.
- **Reproduction:** visit `/onboarding` as new user; view assignment SMS section in Delivery.
- **Impact:** Small spec drift.
- **Verification:** static (a).

---

## Tests

### PRB-060 — Vacuous / mislabelled E2E assertions mask spec gaps
- **Severity:** MAJOR
- **Statement:** Tests pass whether or not the AC holds: T:184 "unregistered phone rejected" (asserts URL stays `/login`, same as invite step); T:390 "guidelines" (asserts `season-theme`); T:398 timeline (theme only); T:417 branch update (POM skips fill when inputs absent, `end2end/tests/fixtures/mail_club_page.ts:161`); T:503 countdown (visibility only); T:649 swap (asserts `cycle-visualization` visible; POM `mail_club_page.ts:483-498` waits for any POST status; `action-error` never asserted); T:833 cancelled state (regex `/no season|немає сезону|SMS/` matches generic and cancelled copy, `uk.json:45`; testid `season-cancelled` never asserted); T:885 deactivated sign-in (outcome identical to unknown phone). T:708/720 assert truthiness, never recipient ≠ self or correct pair.
- **Evidence:** `evidence/a-scope.md` Gap 6 and rows cited; `evidence/i-swapverify.md` Q4.
- **Reproduction:** break the behaviour under each test (e.g. remove the cancelled branch) → test still passes.
- **Impact:** Green suite certifies behaviour that is missing (PRB-030, PRB-041, PRB-045, PRB-053).
- **Verification:** static (a, i).

### PRB-061 — Auth hardening has no automated coverage; documented unit tests do not exist
- **Severity:** MAJOR
- **Statement:** No unit or E2E test for OTP expiry, 3-attempt cap, per-phone rate limits, session expiry, logout session-row deletion (no replay of old cookie). `auth.rs` tests (7) cover only `sha256_hex` and constant-time compare. E2E runs in `SAMETE_TEST_MODE` which fixes the OTP and disables limits and deadline gates, so these paths are unreachable from E2E. `guidance/dev-protocol.md` "What to Test Where" claims unit tests for OTP generation/hashing/verification/expiry/rate limiting and session lifecycle.
- **Evidence:** `evidence/a-scope.md` rows 1.2, 1.2 (ARCH), 1.4, Gap 5; `evidence/e-docs.md` #53, F4.
- **Reproduction:** `grep -n "#\[test\]" -A2 src/auth.rs`.
- **Impact:** Security-critical paths can regress unnoticed; docs overstate coverage.
- **Verification:** static (a, e).

### PRB-062 — Implemented behaviours with no test
- **Severity:** MINOR
- **Statement:** 27 trace rows are IMPLEMENTED-UNTESTED, including: invite distributor recorded; deactivated phone cannot re-register; used code cannot be revoked; invite filter; receipt-nudge count; nudge targeting of non-responders/unconfirmed only (both tested only before any response/confirm); deactivated exclusion from season-open SMS; 5.1 SMS body and once-only; single-use under concurrency; cohort split and >15 pool in E2E; admin-only server checks for cancel; create-season negative validation (past/order).
- **Evidence:** `evidence/a-scope.md` trace table (OK-U rows) and coverage counts.
- **Reproduction:** n/a (absence).
- **Impact:** Regressions in these behaviours undetected.
- **Verification:** static (a).

### PRB-063 — No geometric visual assertions
- **Severity:** MINOR
- **Statement:** Zero clip/overflow/overprint/`scrollWidth <= innerWidth` checks in the three specs; agent review of screenshots is the only geometric gate. Component-evaluation criterion E1 is not automated.
- **Evidence:** `deferred_items.md` (first flagged 2026-06-25); `evidence/d-ops.md` deferred re-verify (`scrollWidth|innerWidth|getBoundingClientRect` = 0).
- **Reproduction:** `grep -rnE "scrollWidth|innerWidth|getBoundingClientRect" end2end/tests`.
- **Impact:** Layout regressions at 375px pass CI.
- **Verification:** static (d re-verified).

### PRB-064 — `cohort-seed.sql` uses untargeted `ON CONFLICT DO NOTHING`
- **Severity:** MINOR
- **Statement:** 5 bare `ON CONFLICT DO NOTHING` clauses (`end2end/tests/fixtures/cohort-seed.sql:32,48,63,74,90`) swallow any constraint conflict, not only the intended one.
- **Evidence:** `deferred_items.md` (first flagged 2026-07-11); `evidence/d-ops.md` deferred re-verify.
- **Reproduction:** `grep -n "ON CONFLICT" end2end/tests/fixtures/cohort-seed.sql`.
- **Impact:** Seed errors can be silently masked.
- **Verification:** static (d).

---

### PRB-101 — `deactivateParticipant` waits on a page-wide testid (intermittent E2E failure)
- **Severity:** MAJOR
- **Statement:** `deactivateParticipant` (`end2end/tests/fixtures/mail_club_page.ts:415`) waits on the page-wide `inactive-status` testid, not the clicked row's. `mail_club.spec.ts:881` has the same unscoped check. In full mode an earlier test (`mail_club.spec.ts:880`) has already deactivated one participant, so `inactive-status` already exists when the visual-audit deactivation clicks. The wait either passes on the old element and so never waits, or matches 2 elements and fails strict mode.
- **Evidence:** T1 implementer E2E run 2 of 4: `visual-audit.spec.ts:800` strict-mode violation, `inactive-status` matched 2 rows. Root cause by the T1 spec reviewer (2026-10-07, `orchestration/reviews/spec-worktree-agent-a4d2369a7e8477b86-2340.md`).
- **Reproduction:** run `isolated-capture.sh <s> full` repeatedly; it fails intermittently, about 1 in 4.
- **Impact:** Flaky suite and a vacuous wait that certifies nothing; it blocks the 3-consecutive-green rule.
- **Verification:** empirical (T1 run) + static (T1 spec review).

## Ops/Deploy

### PRB-070 — No first-admin bootstrap in production
- **Severity:** BLOCKER
- **Statement:** The only `INSERT INTO users` paths are `register_with_code` (role defaults `participant`) and `seed/test_admin.sql` (fixed public test phone `+380670000001`, run by `just db-seed`). Registration needs an invite code; invite codes need an admin (`require_admin`, `src/admin/invite_codes.rs:90`; `distributor_id NOT NULL`). README/justfile/spec document no bootstrap.
- **Evidence:** `evidence/d-ops.md` Data table, F5; `evidence/g-authverify.md` Q4.
- **Reproduction:** fresh DB, boot app → no way to obtain an admin through the app.
- **Impact:** Fresh prod DB unusable without manual SQL; running the test seed in prod installs a publicly known phone as admin (trivially exploitable with PRB-022).
- **Verification:** static (d, g).

### PRB-071 — Deploy artifact never built anywhere before Coolify
- **Severity:** MAJOR
- **Statement:** CI never runs `docker build`. Dockerfile installs unpinned `cargo-chef cargo-leptos` on `rust:1.91-slim` (`Dockerfile:2`; slim lacks pkg-config/libssl-dev; tailwind/wasm-opt/wasm-bindgen downloaded at build time). With PRB-001, `Dockerfile:11` would fail anyway.
- **Evidence:** `evidence/d-ops.md` Run/Deploy table + F6; `.github/workflows/ci.yml` (no docker step).
- **Reproduction:** `docker build .` (not run: docker daemon absent in container, d forbidden to run).
- **Impact:** First production deploy is the first test of the image.
- **Verification:** static (d); unverified empirically.

### PRB-072 — Image runs with `Env::DEV`
- **Severity:** MINOR
- **Statement:** `LEPTOS_ENV` unset in Dockerfile; `Cargo.toml:122` `env = "DEV"`. No current consumer of `Env::DEV` in leptos 0.8.17 / leptos_axum.
- **Evidence:** `evidence/d-ops.md` table row `LEPTOS_ENV`, F9.
- **Reproduction:** inspect `Dockerfile` ENV lines.
- **Impact:** Latent: any future DEV-gated behaviour ships to prod.
- **Verification:** static (d).

### PRB-073 — Image and operations lack production basics
- **Severity:** MINOR
- **Statement:** No brotli pre-compression in image (on-the-fly `CompressionLayer`, `main.rs:135`); `cargo chef cook --release` builds default features (`default=[]`) not `ssr` → dep cache miss (`Dockerfile:9`); no HEALTHCHECK; no backups; no security headers; Postgres provisioning in Coolify undocumented (compose is dev-only, `docker-compose.yml:1-14`); `.env.example` omits `TURBOSMS_TOKEN`/`TURBOSMS_SENDER`.
- **Evidence:** `evidence/d-ops.md` Run/Deploy table, env table, F10.
- **Reproduction:** read `Dockerfile`, `docker-compose.yml`, `.env.example`.
- **Impact:** Slower builds, no liveness signal, no recovery path for data loss.
- **Verification:** static (d).

---

## Docs/Claims

### PRB-080 — Recorded project status is false
- **Severity:** MAJOR
- **Statement:** `codebase_state.md` "campaign SHIPPED — origin/main @ c472bfc, CI GREEN" (run 30162383205) — the run skipped Playwright after a compile error (PRB-001, PRB-002). E2E section "119 tests (75+43+1)" and "117/119 pass" — source has 118 (75 + 42 + 1). Status claims were recorded without inspecting job logs. Module inventory omits `components/*`; "62→68" unit-test note stale.
- **Evidence:** `readiness.md` §1–§2; `evidence/m-civerify.md`; `evidence/e-docs.md` #1, #6, #7, #19.
- **Reproduction:** `grep -cE "\btest\(" end2end/tests/*.spec.ts`; `gh run view 30162383205 --log | grep -c "passed"` → 0.
- **Impact:** Next ARRIVE trusts a shipped/green state that does not exist.
- **Verification:** empirical (m CI logs) + static (e counts).

### PRB-081 — README is false or incomplete on env vars, dependencies, structure
- **Severity:** MAJOR
- **Statement:** Env table documents `CSRF_SECRET` ("generated at startup if absent") — read nowhere (`grep env::var` = 9 vars, none CSRF). Omits `SAMETE_TEST_MODE`, `SAMETE_SMS_DRY_RUN`, `SAMETE_LOG_POOL`, `RUST_LOG`. Dependencies list blake2 0.10 "OTP/session hashing, CSRF" — no blake2, no CSRF code; hashing = sha2 + subtle; omits subtle, base64, leptos-use, leptos_i18n. uuid features "v4, serde, js" (`js` only via hydrate). Project tree omits `invite_codes.rs`. Recipe table omits `capture-isolated`, `fmt`, `fmt-check`.
- **Evidence:** `evidence/d-ops.md` env table, F7; `evidence/e-docs.md` #5, #20, #22, #24, F3.
- **Reproduction:** `grep -rn "blake2\|csrf\|CSRF" Cargo.toml src` → 0; `grep -rn "env::var" src`.
- **Impact:** Operator configures prod from a table that hides the two most dangerous flags (PRB-022, PRB-023).
- **Verification:** static (d, e).

### PRB-082 — CLAUDE.md and E2E guide stale on counts, commands, timeouts
- **Severity:** MINOR
- **Statement:** CLAUDE.md "`just test` 55 bare / 62 ssr" — actual 68/75. CLAUDE.md "`just clippy` Run clippy (SSR)" — runs SSR and wasm32 hydrate. `navigationTimeout` "15s" in CLAUDE.md, `end2end/README.md`, `mail_club_page.ts:136` comment — `end2end/playwright.config.ts:38` = `30_000`.
- **Evidence:** `evidence/e-docs.md` #1, #3, #9, F5–F7; `evidence/b-build.md` finding 3.
- **Reproduction:** `grep -n navigationTimeout end2end/playwright.config.ts`; `grep -n "^clippy" -A3 justfile`.
- **Impact:** Agents reason from wrong numbers (timeout budget, test baseline).
- **Verification:** static (e) + empirical counts (b).

### PRB-083 — Guidance ban rules omit the exceptions present in code; z-index map aspirational
- **Severity:** MINOR
- **Statement:** `waitForTimeout` banned (e2e README, conventions) but `paintSettle` uses it (`end2end/tests/fixtures/capture-constants.ts:40`, sanctioned in comment `:35`). `!important` banned (frontend-protocol) but used twice (`style/components.css:112-113`, reduced-motion block, commented). frontend-protocol z-index map lists dropdown 40 / modal 50 / confetti 60; only grain `z-index: 1` exists in `style/`.
- **Evidence:** `evidence/e-docs.md` #12, #40, #41, F8, F9.
- **Reproduction:** `grep -rn "waitForTimeout" end2end/tests`; `grep -rn "important\|z-index" style/`.
- **Impact:** Binding guidance and code disagree; reviewers flag sanctioned code or trust nonexistent layers.
- **Verification:** static (e).

### PRB-084 — Spec documents contradict each other and the code
- **Severity:** MAJOR
- **Statement:** (a) PS:30 "No self-registration… organizer manually signs them up" and PS:150 "Registration (organizer creates account)" vs US 1.1/1.5 invite-code self-registration (implemented, `login.rs:382`). (b) US 4.2 (`US:276`) "Launch triggers SMS notification" and US 5.4 (`US:442`) "Timed ~1 hour before the ready-confirm deadline" vs ARCH:37 "Organizer triggers all SMS batches… No cron, no schedulers" (code follows ARCH). (c) ARCH:161 "Deactivated accounts are still rejected at the OTP request stage" vs US + code (`login.rs:199-206` sends OTP, redirects silently after verify). (d) ARCH:169/171 lockout and global cap, ARCH:518 advance gated on confirm deadline, ARCH:520 per-participant receipt status — none implemented (PRB-024, PRB-031, PRB-042).
- **Evidence:** spot-checked PS:30, PS:150, US:276, US:442, ARCH:37, ARCH:161, ARCH:167-171; `evidence/a-scope.md` rows 1.1 (deactivated), 4.2, 5.4, "PS Entry", Gap 9; `evidence/j-feel.md` O4.
- **Reproduction:** read the cited lines side by side.
- **Impact:** No single authoritative requirement; implementers and reviewers pick different sources; ACs cannot be judged pass/fail.
- **Verification:** static (a, author spot-check).

### PRB-085 — `deferred_items.md` contains wrong entries and misses every blocker
- **Severity:** MINOR
- **Statement:** (a) Panic item attributed to "intermittent `tower_http` 500s" — it is a detached-task process abort (PRB-010); the 500 lines are expected server-fn errors. (b) "implementation_plan.md needs keep/archive/delete" — file does not exist in `orchestration_log/reference/` (`archive/spec/implementation-plan.md` exists). (c) "create-form unreachable once any season row exists" — contradicted: `AfterTerminalSeason` form at `page.rs:803`, exercised by T:801. (d) cfg WHY-comment item says 10 sites; there are 12. (e) None of PRB-001, -002, -010, -020, -022, -030, -070 is listed.
- **Evidence:** `evidence/n-panic.md` "Root cause" last paragraph; `evidence/d-ops.md` deferred re-verify, F12; `evidence/e-docs.md` #50; `evidence/a-scope.md` row 4.5; `readiness.md` §1.
- **Reproduction:** `ls orchestration_log/reference/`; read `deferred_items.md`.
- **Impact:** The live debt list misdirects the next session.
- **Verification:** static (d, e, a) + empirical (n for panic attribution).

### PRB-086 — `#[cfg(any(feature = "ssr", test))]` gate has no orienting comment
- **Severity:** MINOR
- **Statement:** 12 sites (`phone.rs`, `invite_codes.rs`, `assignment.rs`, `home.rs`) use the gate without a WHY comment (e.g. `phone.rs:14`).
- **Evidence:** `deferred_items.md` (first flagged 2026-07-12); `evidence/d-ops.md` deferred re-verify; `evidence/b-build.md` finding 2 (12 sites).
- **Reproduction:** `grep -rn 'cfg(any(feature = "ssr", test))' src`.
- **Impact:** Future module authors misplace pure code behind the ssr gate (cause of the 2026-07-12 invisible-tests finding).
- **Verification:** static (d, b).

---

## Environment/Tooling

### PRB-090 — No reproducible environment bootstrap
- **Severity:** MAJOR
- **Statement:** Fresh cloud container lacks cargo-leptos, just, sqlx-cli, cargo-audit, brotli, `end2end/node_modules`, a running Postgres, and a docker daemon (CLAUDE.md/conventions assume Docker Postgres). No SessionStart hook (`.claude/settings*.json` absent), no setup script, no `rust-toolchain` file. Every agent re-derives setup (binstall, native `postgresql-16` cluster, role/db creation, `npm ci`, browser shim, compiler overrides).
- **Evidence:** `evidence/b-build.md` "E2E runnability"; `evidence/f-e2e.md` Setup table; spot-check `ls .claude/settings*.json` → absent; orchestrator report "cargo-leptos absent from container".
- **Reproduction:** new session: `which cargo-leptos just sqlx; pg_isready; docker info`.
- **Impact:** Every session pays setup cost; environments drift between agents and from CI; E2E not runnable out of the box.
- **Verification:** empirical (b, f).

### PRB-091 — cargo-leptos version resolution diverges from CI
- **Severity:** MAJOR
- **Statement:** `cargo binstall cargo-leptos` resolves 0.3.11 now; GitHub GraphQL lookup 403s through the agent proxy, so binstall falls back to a source build. CI's last run used 0.3.7. Version chosen depends on date and network path.
- **Evidence:** `evidence/f-e2e.md` Setup row cargo-leptos, Environment deltas; `logs/f-e2e-run1-rustc1.97.1-leptos0.3.11-FAIL.log`; `logs/f-setup.log`.
- **Reproduction:** `cargo binstall cargo-leptos --no-confirm` in the container; observe resolved version and fallback.
- **Impact:** Local builds use a different build tool than CI.
- **Verification:** empirical (f).

### PRB-092 — Playwright browser revision mismatch with preinstalled browsers
- **Severity:** MAJOR
- **Statement:** `end2end/package.json` pins `"@playwright/test": "^1.58.2"` (caret range), which expects `chromium-1208` / `chromium_headless_shell-1208` with `chrome-headless-shell-linux64` layout. `/opt/pw-browsers` ships revision 1194 (`chrome-linux`, binary `headless_shell`). Runs only via hand-made symlink directories (present now: `/opt/pw-browsers/chromium-1208/chrome-linux64 -> …/chromium-1194/chrome-linux`). A scratchpad shim pointing at a nonexistent path broke one run.
- **Evidence:** `evidence/f-e2e.md` Setup row Chromium; `evidence/n-panic.md` A/B diff row Playwright; `logs/n-run0-browserpath-FAIL.log`; spot-check `ls -la /opt/pw-browsers/chromium-1208/`.
- **Reproduction:** fresh container: `cd end2end && npm ci && npx playwright test` → browser executable not found.
- **Impact:** E2E cannot start without out-of-repo filesystem surgery.
- **Verification:** empirical (f, n; author observed shim).

### PRB-093 — cargo-leptos-downloaded tailwindcss "musl" binary reported unrunnable
- **Severity:** MINOR
- **Statement:** Orchestrator reports the tailwindcss musl binary fetched by cargo-leptos is unrunnable in this container. At authoring time `/root/.cache/cargo-leptos/tailwindcss-v4.2.1/tailwindcss-v4.2.1/tailwindcss-linux-x64-musl --help` exits 0 and `file` reports a glibc-dynamic ELF (`interpreter /lib64/ld-linux-x86-64.so.2`) despite the `-musl` name — the cached file may have been replaced during the session.
- **Evidence:** orchestrator environment report; author check (output: `≈ tailwindcss v4.2.1`, exit 0).
- **Reproduction:** clear `~/.cache/cargo-leptos/tailwindcss-*`, run `cargo leptos build`, observe tailwind step.
- **Impact:** CSS build step fails in fresh containers (when it occurs).
- **Verification:** reported by orchestrator; not reproduced at authoring time.

### PRB-094 — Disk allowance exhausts under parallel cargo builds
- **Severity:** MAJOR
- **Statement:** Shared `target/` under concurrent agent builds reached 14 GB in `target/debug`; `/` hit 100% twice in f and once in l, producing ENOSPC, link bus errors, rmeta write failures, and invalid bisect results. Free space at authoring: 11 GB of 252 GB (73% used).
- **Evidence:** `evidence/f-e2e.md` Environment deltas (Disk); `evidence/l-toolchain.md` Notes; `logs/f-e2e-run1-enospc.log`, `logs/f-e2e-run1-enospc2.log` (`ENOSPC` in `cached-context.ts:91`); author `df -h /`.
- **Reproduction:** run two release + wasm builds in separate worktrees concurrently with separate target dirs.
- **Impact:** Parallel agent work fails nondeterministically; corrupted build evidence.
- **Verification:** empirical (f, l).

### PRB-095 — Platform worktrees start from stale commits and vanish on idle stop
- **Severity:** MAJOR
- **Statement:** Agent worktrees are created from an old commit, not the current branch tip, and are deleted when the agent idles/stops (including session-limit pauses). After deletion, a resumed agent's cwd falls back to the main repo and commits land on the main branch unreviewed.
- **Evidence:** orchestrator environment report; `git worktree list` at authoring: branch tip `5360af1`, agent worktrees at `57f0109`, `fd94520`, `d2429d3`, `066ebf2`, `294db28`, `108aa7d`; `orchestration_log/reference/conventions.md` "Added 2026-07-15" (admin-leaves worktree destroyed → 3 commits on main).
- **Reproduction:** spawn an isolated-worktree agent; `git -C <worktree> log -1` vs branch tip; let agent idle; `git worktree list`.
- **Impact:** Agents implement against outdated code; work and isolation lost; review gate bypassed.
- **Verification:** reported by orchestrator; worktree base drift observed by author; destruction documented in conventions.

### PRB-096 — Sibling test databases accumulate
- **Severity:** MINOR
- **Statement:** 13 `samete*` databases exist on the container cluster (`samete_u1 _u3 _u4 _fattr _pm1 _pm2 _ph1 _ph2 _ph3 _pr _pf _e2e_pv_1` + `samete`); `samete_fattr` belongs to the finished f-e2e run. Deferred item: orphan `samete_ssr_debug2` on the owner's machine (not checkable here). No harness asserts cleanup.
- **Evidence:** author `psql … pg_database` query; `deferred_items.md` (first flagged 2026-07-10); `evidence/n-panic.md` cleanup note (n dropped its own).
- **Reproduction:** `PGPASSWORD=samete psql -h localhost -U samete -d samete -Atc "select datname from pg_database where datname like 'samete%'"`.
- **Impact:** Disk use and confusion over which DB is live. (Some listed DBs may belong to agents still running.)
- **Verification:** empirical (author); owner-machine item unverified.

### PRB-097 — Clone is shallow by default
- **Severity:** MINOR
- **Statement:** Container clone holds ~50 commits; `git cat-file` rejects run SHAs `6fc68ca`, `6600644`, `36e62e5`, `80d1c70`. Bisect required `git fetch --unshallow origin`.
- **Evidence:** `evidence/c-ci.md` §3 last bullet; `evidence/l-toolchain.md` Notes.
- **Reproduction:** fresh session: `git rev-list --count HEAD`.
- **Impact:** History-dependent investigation (bisect, CI-run matching) fails until unshallowed.
- **Verification:** empirical (c, l).

---

### PRB-102 — Unpinned rustc drifted to 1.99; wasm clippy gate red repo-wide
- **Severity:** MAJOR
- **Statement:** The container toolchain moved 1.97 → 1.99.0 mid-campaign. Under 1.99, `cargo clippy --target wasm32-unknown-unknown --features hydrate -- -D warnings` fails with 31–32 errors on untouched code. 29–30 come from the new pedantic `unused_async_trait_impl` lint on every `#[server]` expansion; 2 are `.ok().is_some_and` in `src/pages/login.rs` (~588/608). CI still runs 1.97.x, so local and CI gates disagree.
- **Evidence:** reported independently by the U4, T0, U1 and T1 implementers and by the U1 and T1 spec reviewers (2026-10-06/07).
- **Reproduction:** `rustc --version` → 1.99.0; run the command above.
- **Impact:** A standard gate is red on every branch; every lane must carve it out. This is a concrete consequence of PRB-090 (no `rust-toolchain` pin).
- **Verification:** empirical.

## Process

### PRB-100 — Manifesto SubagentStart hook injects no constitution elements
- **Severity:** MINOR
- **Statement:** The SubagentStart hook reports "No role-specific elements matched" and injects zero elements; binding depends on each dispatch carrying full manifesto paths. `/tmp` manifesto repo can be absent at session start.
- **Evidence:** `deferred_items.md` (first flagged 2026-07-03); `conventions.md` "Added 2026-07-12"; observed again in this agent's own SubagentStart context.
- **Reproduction:** spawn any subagent; inspect injected hook context.
- **Impact:** Every dispatch carries binding boilerplate; an omitted path yields unbound output.
- **Verification:** empirical (author, this session).

---

## Coverage

### Source → PRB map

| Source § | PRB IDs |
|---|---|
| a-scope trace rows 1.1 (failure paths, single-use concurrency, distributor, deactivated re-register, deactivated redirect, codes never expire) | 027, 062, 084, 021 |
| a 1.2, 1.2 (ARCH) OTP + lockout/global cap | 061, 024 |
| a 1.3 / 2.1 branch update | 041 |
| a 1.4 logout (onboarding, session deletion) | 057, 061 |
| a 1.5 / 1.6 invite codes (list, filter, revoke-used) | 062 |
| a 2.1 deadline, timeline, guidelines | 051, 045, 052 |
| a 2.2 enrolled precondition, countdown | 040, 053 |
| a 2.3 truthiness asserts, SMS pre-release | 060, 034 |
| a 2.4 5-day prompt, nudge targeting, note invisible | 044, 062, 042, 048 |
| a 3.1 generation guard/tx | 032 |
| a 3.2 no DB-integrated test | 062 |
| a 3.3 swap | 030, 060 |
| a 4.1 negative validation, theme in Preparing | 062, 052 |
| a 4.2 launch SMS | 056, 084 |
| a 4.3 cancelled-state assertion, cancel pre-launch, admin-only check | 060, 035, 062 |
| a 4.4 counts (M only, receipt-nudge untested) | 057, 062 |
| a 4.5 create-form stale deferred item | 085 (and rejected list) |
| a 4.7 advance unguarded, ARCH:518 | 031 |
| a 5.1–5.4 SMS body/once/targeting, timing | 062, 056 |
| a 6.1 deactivation (self guard, sign-in test, exclusion) | 028, 060, 033 |
| a PS rows (self-registration, failure protocol, >15 cohort E2E) | 084, 042, 062 |
| a Gaps 1–9 | 030, 041, 042, 040, 061/022, 060, 045/052/053, 035/032/034/031/033/028, 084/085 |
| a Scope creep | rejected list |
| b-build checks + findings 1–5, E2E runnability | 004, 082, 007, 008, 090 |
| c-ci §1–§2 runs, §3 HEAD coverage, §4 red runs, §5 PRs, findings | 002 (supersedes "green"), 006, 007, 097, rejected list |
| d-ops Run/Deploy table | 071, 072, 073 |
| d env table | 022, 023, 081, 073 |
| d security table, test-mode leakage | 020, 022, 023, 024, 025, 026 |
| d Data table (first admin, backups) | 070, 073 |
| d deferred re-verify | 010, 100, 024, 063, 086, 096, 064, 085 |
| d F1–F12 | 020/021, 022, 024, 025, 070, 071, 081, 023, 072, 073/026, 085 (F11 rejected), 085 |
| e-docs #1–#55 | 080, 082 (#1, #3, #9), 081 (#5, #20, #22, #24), 080 (#6, #7, #19), 083 (#12, #40, #41), 050 (#44), 085 (#50), 061 (#53), 005 (#28); TRUE rows and unverifiable rows → rejected list |
| e F1–F11 | 050, 004, 081, 061, 080, 082, 082, 083, 083, 085/081, rejected list |
| f-e2e Setup, Environment deltas | 090, 091, 092, 094, 005 |
| f Runs / Failures | 010, 094 |
| f Finding 1 | 001 |
| f attr: leak check | 050 |
| f Auth bypass empirical check | 020 |
| g-authverify Q1a/1c, Q1b, Q2, Q3, Q4, other trust edges | rejected (1a, 1c), 020, 021/027, 026/022/023, 070, 025/024 |
| i-swapverify Q table, math, domain rule, hidden-bug note, unit test | 030, 031, 060 |
| j-feel Participant P1–P14 | 043, 044, 040, 051, 052, 054, 054, 054, 054, 045, 046, 055, 053, rejected (P14) |
| j Organizer O1–O9 | 042, 047, 048, 056, 056, 049, 054, rejected (O8, O9) |
| j Candidates C1–C18, Top 8 | solution proposals → not problems (rejected list); their problem evidence is covered by the P/O rows above |
| l-toolchain root cause, reproduction, boundary, crate roots, CI guard, notes | 001, 002, 003, 005, 094, 097 |
| m-civerify table, findings, gate analysis | 002, 003, 005 |
| n-panic verdict, evidence, A/B diff, repro, root cause, production exposure | 010, 092, 085 |
| readiness §1–§2 declared vs actual | 001, 002, 010, 020, 030, 080, 081, 082, 083, 050, 061, 085 |
| readiness §3 B1–B7 | 001/003, 002, 010, 020/021, 030/031, 022, 070 |
| readiness §4 Majors | 050, 041, 042, 040, 043, 044, 045, 046, 047, 060, 024, 025, 071, 081, 023, 004, 048/049 |
| readiness §4 Minors | 082/080, 081, 085, 035, 032, 031, 033/028, 054, 051/052/053, 072/073/026, 007, 090/091/092/094, 083 |
| readiness §5 solid, §6 product read, §7 confidence | no problems (§6 restates 040–047); §7 unverified items → 071, 007, 010, 093 |
| deferred_items: SSR panic | 010, 085 |
| deferred_items: oath hook | 100 |
| deferred_items: IP OTP rate limit | 024 |
| deferred_items: geometric assertions | 063 |
| deferred_items: cfg WHY comment | 086 |
| deferred_items: orphan DB `samete_ssr_debug2` | 096 |
| deferred_items: cohort-seed ON CONFLICT | 064 |
| deferred_items: create-form unreachable | rejected (see below); stale entry → 085 |
| deferred_items: implementation_plan.md decision | rejected (file absent); stale entry → 085 |
| Env report: cargo-leptos absent | 090, 091 |
| Env report: Playwright 1.58 / chrome-headless-shell layout | 092 |
| Env report: tailwindcss musl unrunnable | 093 |
| Env report: worktrees from old commit, deleted on idle | 095 |
| Env report: disk exhausts under parallel builds | 094 |
| Env report: no toolchain bootstrap / SessionStart hook | 090 |
| Env report: CI rustc unpinned | 005 |
| Env report: e2e harness exits 0 on build failure | 002 |
| Env report: spec docs contradict (SMS triggers; PS no self-registration) | 084 |

### Reported items rejected as non-problems

| Item | Source | Reason |
|---|---|---|
| Create-form unreachable after any season row exists | deferred_items, d F11 | Contradicted by code + test: `AfterTerminalSeason` form at `src/admin/page.rs:803`, exercised by T:801 (a row 4.5; readiness §2 resolution). The stale entry is PRB-085 |
| `implementation_plan.md` keep/archive/delete decision | deferred_items | File does not exist in `orchestration_log/reference/`; nothing to decide. Stale entry is PRB-085 |
| "Toolchain drift" as cause of build failure | f Finding 1 | Superseded by l bisect: source growth at `ee816fd` on a fixed toolchain. Problem kept as PRB-001 with corrected cause |
| "Intermittent tower_http 500s" | deferred_items, f | Those 500s are expected server-fn error responses (n); real defect is PRB-010 |
| cargo-leptos `ProcessHandle should not have been dropped` panic | f logs | Harness teardown after server death, not a second bug (n) |
| Swap "persists corrupted graph" | a Gap 1 | Superseded by i: UNIQUE violation aborts the first UPDATE; latent persistence risk recorded inside PRB-030 |
| Red CI runs 29188964277, 28171533086 | c §4 | Each fixed by the next push; no residual defect |
| Existing-phone login without OTP; takeover via forged cookie | g 1a, 1c | Verified not possible (OTP required; `users.phone UNIQUE`) |
| Cancelled season sends no SMS | j P14 | Declared behaviour (US 4.3) |
| Known groups editable only via SQL | j O8 | Declared design (PS App Scope "organizer edits database directly") |
| No per-participant season history / deactivation reason | j O9 | Declared deferred in PS §Deferred "Season history" |
| Scheduled SMS cron; RSVP / meetup scheduling | j Rejected | Excluded by ARCH:37 and PS; not defects |
| j candidates C1–C18, l/n/i/g/d "fix" sections | j, l, n, i, g, d | Solution proposals, not problems; underlying problems registered |
| Scope creep (resend cooldown, stepper, toast, skeletons, dark mode, grain, EnrollmentNotOpen, cycle-viz scaling, invite filter count, idempotency tables, capture tooling) | a | Implemented without spec backing; no defect observed |
| "Legal name matches government ID" | a row 1.1 | Not checkable in code; spec process requirement |
| Design-token, module-inventory, Phase-enum, CI-shape, dependency-version, recipe-existence claims marked TRUE | e #2, #4, #5 (existence), #8, #10, #11, #13–#17, #21, #23, #25, #26, #29, #31–#39, #42, #45–#49 | Verified true |
| Unverifiable-statically claims: WASM sizes, Tailwind 4.2.1, MCP tool names, bridge files, `.manifestos.yaml` hook, hydration gate on all 23 submit sites (1 sampled TRUE) | e #18, #27, #43, #54, #55; #28 partly → PRB-005 | No defect evidence; #55 hook gap is PRB-100 |
| tailwind.css "1626 lines" history | e #30 | Historical note, not current state |
| d OK rows (binary path, site root, migrations, TLS, cookie flags, admin guards ×18) | d | Verified sound |
| Reverse proxy HTTPS requirement | d | Claimed provided by Coolify/Traefik; no defect evidence |
