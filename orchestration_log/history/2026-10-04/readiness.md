# Readiness verdict — Саме Те Mail Club (2026-10-04, HEAD 1a3c8a5; plan commits to 330fc89 are docs-only)

Sources: reports in `orchestration_log/recon/2026-10-04/readiness/` (**gitignored** — recon is disposable; cited by filename below) and tracked plans `orchestration_log/history/2026-10-04/plans/PLAN-W1.md`, `PLAN-W2.md`. Report key: a=a-scope, b=b-build, c=c-ci, d=d-ops, e=e-docs, f=f-e2e, g=g-authverify, i=i-swapverify, j=j-feel, l=l-toolchain, m=m-civerify, n=n-panic. No fix unit is implemented yet (git log: plans only).

## 1. Verdict

**NOT READY.** HEAD does not build a deployable binary (B1). CI has been false-green since 2026-07-10 (B2). Concurrent load kills the server (B3). Registration skips OTP (B4, live-reproduced).
- codebase_state "SHIPPED, CI GREEN": **false**. Zero Playwright tests ran in any readable CI log (m, l).
- spec "All 6 phases complete": **true for the phase enum (e#21), false for stories**. 3 MISSING, 2 code-CONTRADICTED, 10 PARTIAL of 95 AC rows (a). Swap (3.3) never works (B5).
- CLAUDE.md / README: largely accurate on design tokens, modules, versions and recipes (e). Stale on test counts, timeouts and env vars; README is false on blake2/CSRF.
- deferred_items: 2 entries wrong (misattributed panic; nonexistent file). It is missing every blocker below.

## 2. Declared vs actual

| Claim | Source | Actual | Evidence |
|---|---|---|---|
| CI green @ c472bfc, run 30162383205 | codebase_state.md | E2E job hit `queries overflow the depth limit!` and skipped Playwright, yet concluded success. Same in every readable run 07-10..07-25 | m, l (supersede c "CI green") |
| Campaign shipped | codebase_state.md | Release bin + wasm lib fail to compile at HEAD (rustc 1.97.0/1.97.1/1.99.0). Trigger commit ee816fd (07-09) | l (bisect), f logs |
| 117/119 E2E pass | codebase_state.md | Suite has 118 tests. Once built (recursion workaround): 116 pass / 2 skip on the isolated harness, but server crash in 4/5 cargo-leptos-path runs | e#6, n repro table, f runs |
| `just test` 55 bare / 62 ssr | CLAUDE.md | 68 bare / 75 ssr, all pass. CI runs bare only, so 7 auth tests never run in CI | b, e#1 |
| All 6 phases complete | spec/, dev-protocol.md | Enum yes; stories no (see §1) | e#21, a coverage table |
| Swaps preserve single loop (3.3) | User Stories.md | Swap can never succeed: first UPDATE violates non-deferrable UNIQUE. If bypassed, any in-cycle swap splits the loop. E2E T:649 is vacuous | i (static), a Gap 1 |
| Deactivated / leptos panic = "intermittent tower_http 500s" | deferred_items.md | Misattributed: detached-task panic in leptos_i18n 0.6.1 `context.rs:213`; `panic="abort"` makes it process death | n (supersedes f's attribution) |
| Create-form unreachable after any season exists | deferred_items.md, d F11 | Form exists (`AfterTerminalSeason`, admin/page.rs:803); E2E T:801 exercises it and passes in green runs | a row 4.5 (code+test) > d (static ordering read) |
| `implementation_plan.md` decision pending | deferred_items.md | File does not exist | d F12, e#50 |
| `CSRF_SECRET` env var, blake2 for hashing/CSRF | README.md | Neither exists; hashing = sha2 + subtle | d env table, e#22 |
| README env table complete | README.md | Omits SAMETE_TEST_MODE / SAMETE_SMS_DRY_RUN | d F7 |
| navigationTimeout 15s | CLAUDE.md, e2e README | 30_000 | e#9 |
| All submit buttons `aria-busy` | codebase_state 2026-06-22 | 13 admin buttons emit literal `attr:aria-busy` while pending (DOM-confirmed) | f attr: leak check (supersedes e F1 "unverifiable") |
| `waitForTimeout` / `!important` banned | e2e README, frontend-protocol | 1 sanctioned waitForTimeout (paintSettle), 2 `!important` in reduced-motion. Documented exceptions, not in the ban text | e#12, e#40 |
| z-index layers 40/50/60 | frontend-protocol | Only grain z-index 1 exists | e#41 |
| Unit tests for OTP expiry/rate-limit/session | dev-protocol.md | None; auth tests cover hash helpers only | e#53, a row 1.2 |

## 3. Blockers (ranked)

| # | What | Evidence | Impact | Planned fix |
|---|---|---|---|---|
| B1 | SSR bin + wasm hydrate lib exceed default `recursion_limit` at codegen. clippy/test do not codegen, so they stay green | l (bisect ee816fd; rl256 passes), f Finding 1, m | No deployable artifact. Coolify build fails the same way (Dockerfile runs `cargo leptos build --release`, d) | **T0** (`#![recursion_limit="256"]` ×2 + CI codegen builds) |
| B2 | E2E false-green: `cargo leptos end-to-end` exits 0 on build failure; no step asserts tests ran | m (4 runs), l | Every "CI green" since 07-10 is unproven; regressions invisible | **T0** (`scripts/assert-playwright-ran.sh` in all e2e recipes) |
| B3 | leptos_i18n SSR effect reads a disposed signal in a detached task; `panic="abort"` kills the server | n (debuginfo backtrace 8/8; stress repro in seconds; unwind binary survives) | Any concurrent burst downs prod (same profile) | **T1** (`panic="unwind"` server, abort kept for wasm + `ssr-stress.sh`) |
| B4 | `pending_phone` cookie is raw, unsigned phone; `register_with_code` trusts it. curl + one invite code = account for any unclaimed phone, no OTP. Invite redemption unthrottled (~39,800 codes) | f auth-bypass (live: 200, session cookie, DB rows), g 1b/Q2, d F1 | OTP auth defeated for registration | **U1** (server-side `registration_tickets` + attempt cap) |
| B5 | Swap (3.3) never succeeds: UNIQUE violation on first UPDATE. Recipient-exchange math always splits a cycle; no tx; validates after writes; one-cohort validator; advance never re-validates | i (static, unexecuted), a Gap 1 | Organizer cannot fix assignments; corrupt graph possible if constraint ever relaxed | **U4** (position-exchange, transactional, re-validate on advance) |
| B6 | `SAMETE_TEST_MODE=true` = OTP `000000` for all incl. admin, rate limits off; no boot guard; `.env.example` ships it on; README silent | d F2, g Q3 | One env misconfig = full auth bypass | **U2** (`SmsMode`; boot refuses test mode unless dry-run + loopback) |
| B7 | No first-admin path in prod; invite codes need an admin (circular) | d F5, g Q4 | Fresh prod DB unusable without manual SQL; test seed would install a public phone as admin | **U3** (`SAMETE_ADMIN_PHONE`/`_NAME` upsert at boot) |

Order per PLAN-W1: T0 → T1 → {U1 ∥ U3 ∥ U4} → {U2 ∥ A1}.

## 4. Majors and Minors

### Majors

| What | Evidence | Planned |
|---|---|---|
| `attr:aria-busy` leak on 13 admin buttons | f attr: leak check | A1 |
| Branch not editable after onboarding (1.3 AC3, 2.1 AC2) | a Gap 2 | W2-H3 |
| Receipt note + non-receiver identity invisible; organizer sees counts only | a Gap 3, j O1 | W2-R |
| Non-enrolled get live confirm-ready CTA (silent no-op); non-participants told "assigning" all delivery | a Gap 4, j P3 | W2-H1 |
| Recipient card vanishes on own receipt confirm | j P1 | W2-H2 |
| "Не отримав" irreversible, offered day 0 | j P2 | W2-H2 (reversible; time-gate dropped, D2) |
| No meetup info anywhere | j P10, a row 2.1 | W2-M1, W2-M2 |
| No site link in SMS | j P11 | W2-S |
| Forwarding protocol computed mentally | j O2 | W2-F |
| Vacuous E2E assertions (T:184/390/398/417/503/649/833/885) | a Gap 6 | W2-V, W2-H3, W2-M2; T:649 → U4 |
| Unauthenticated `request_otp` → SMS-pumping; phone-only limits, no IP/global cap | d F3, a row 1.2-ARCH | unplanned (deferred item) |
| `request_otp` reveals account existence (NewAccount/AccountExists) | d F4, g | unplanned (U2 explicitly excludes) |
| Dockerfile never built in CI; slim image + unpinned cargo-leptos untested | d F6 | unplanned |
| README env table wrong (CSRF_SECRET, missing SAMETE_*) | d F7 | U2.7 |
| Dry-run logs OTP codes at info | d F8 | U2 (boot guard + docs) |
| CI runs bare `cargo test`; 7 ssr tests never in CI | b F2, e F2 | unplanned (plans run `--features ssr` only as local gates) |
| No organizer alert on not-received; season dates/theme not editable | j O3, O6 | unplanned (W2 D15 drops C8/C9) |

### Minors

| What | Evidence | Planned |
|---|---|---|
| Doc counts stale (tests 55/62, E2E 119, navigationTimeout) | b, e#1/#6/#9 | unplanned |
| README blake2/CSRF + missing deps | e#22 | unplanned (U2 removes CSRF row only) |
| deferred_items stale (panic wording, implementation_plan.md, create-form bullet) | n, d F12, a row 4.5 | T1.4 rewrites panic item; rest unplanned |
| Cancel available pre-launch (4.3 CONTR) | a row 4.3 | unplanned |
| Generation DELETE precedes N≥3 guard, no tx | a Gap 8 | unplanned (U4 excludes generation) |
| Advance Assignment→Delivery unguarded server-side | a row 4.7 | U4 (re-validate on leaving Assignment) |
| Deactivated users counted in nudges/pool; no self-deactivate guard | a Gap 8 | unplanned |
| Copy: pen-pal framing, "кілька хвилин", phantom page refs | j P6-P9, O7 | W2-U0 |
| Theme hidden after enroll; enroll form after deadline; no countdown | j P4/P5/P13 | dropped (W2 D15) |
| `LEPTOS_ENV` unset, no pre-compression/healthcheck/backups, unsalted OTP hash, unswept sessions | d F9/F10 | unplanned |
| cargo-audit freshness (last run 07-25) | c, b | unplanned |
| Toolchain-sensitive setup (cargo-leptos 0.3.11, Playwright revision, disk) | f Setup | plans' shared-env recipe (container only) |
| Docs out of sync (frontend-protocol z-index map; ban text missing documented exceptions) | e#12/#40/#41 | unplanned |

## 5. Genuinely solid

| Area | Evidence |
|---|---|
| fmt, clippy SSR + wasm (`-D warnings`), 75/75 unit tests | b (executed) |
| Login for existing phones requires OTP; sessions random, hashed, HttpOnly/Secure/Strict, 90d | g 1a, d security |
| Existing-account takeover via cookie blocked (phone UNIQUE) | g 1c |
| All 18 admin server fns behind `require_admin`; 31/31 `#[server]` accounted | d server-fn table |
| Assignment algorithm: cycles, cohort split (25→13+12), social weighting, unit-tested | a rows 3.1/3.2 |
| Invite codes single-use under concurrency (`FOR UPDATE` tx) | g Q2, a row 1.1 |
| Migrations ordered, non-destructive, embedded, run at boot | d Data |
| Design-system tokens (palette, dark, badges, density, type, radii, grain) match docs exactly | e#31-#39 |
| Once built, full suite 116 pass / 2 skip (isolated harness, 2/3 + 6/6 debuginfo) | n repro table |
| No open PRs/issues; HEAD tree == last CI'd tree (no unvalidated code since) | c |

## 6. Product read (j)

- Core logistics loop works end to end for one small cohort. The app stays "logistics only", as designed.
- Participants hit dead ends: address lost after own receipt, irreversible "not received", false "assigning" promises.
- The payoff (meetup) is absent from the app. SMS have no clickable link.
- The solo organizer runs on counts, not names, and does forwarding by hand. The time sink is DB access.
- W2 covers the top 8 by value/effort (C1–C7, C10). It defers season edit, the not-received alert and the theme persistence fix.

## 7. Confidence

| Class | Items |
|---|---|
| Empirical (executed) | B1 build failure + rl256 fix (l, f); B2 CI logs (m, l); B3 panic, backtrace, unwind survival (n); B4 live bypass (f); attr:aria-busy DOM (f); fmt/clippy/unit tests (b); E2E pass counts (n, f) |
| Static only | B5 swap (i: math + constraint trace, unit test written but not run); B6, B7 (d, g); all a-scope trace rows; d ops/env/security; e doc claims; j product frictions |
| Unverified | Coolify/Dockerfile build (never run); onset before 07-10 (CI logs 410); whether pre-1.97 rustc builds HEAD (l); cargo-audit against current advisories; WASM size claims (471KB); prod env actually set by owner; the subscriber_traits.rs:112 panic variant pinned to i18n by inference only (n); behaviour of the fixes, which do not exist yet |
| Disagreements resolved | c "CI green" → m/l false-green. f "toolchain drift" → l source growth (ee816fd). f "tower_http 500s" → n i18n detached task. d F11 create-form → a row 4.5. a "swap persists corruption" → i "swap errors at UNIQUE first" (both static; i is later and deeper). e F1 aria-busy unverifiable → f confirmed. g 1b static → f confirmed |
