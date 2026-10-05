# Implementation Plan: Every Remaining Finding (F)

Owner directive: every finding gets fixed — no "unplanned", no "deferred". Coverage: §1 matrix maps every row of `readiness.md`, every finding in the raw reports (`orchestration_log/recon/2026-10-04/readiness/{a-scope,b-build,c-ci,d-ops,e-docs,f-e2e,g-authverify,i-swapverify,j-feel,l-toolchain,m-civerify,n-panic}.md`) and every open `reference/deferred_items.md` item to a unit in `PLAN-blockers.md` (T0, T1, U1–U4, A1), `PLAN-product.md` (P-*), or an F-unit below. Where no competent default existed in the reports, this plan picks one and records the rationale (§2).

**Parallelism (binding, `orchestration_log/recon/2026-10-04/fix/prompts/_parallelism.md`):** every F-unit is its own concurrent lane (own worktree, own agent) from `claude/loving-johnson-7l8hn5`. Shared files never order lanes; integration unions JSON keys/POM methods, regenerates `.sqlx/`, re-runs gates. Each unit is reviewed and integrated as soon as it finishes. The one batch: lane **DOC-OPUS** (one opus agent, separate commit per doc unit) — doc units are small text edits, and conventions.md requires opus for agent-instruction docs.

**Build env (every cargo command, every lane):** `export CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0`. Until T0's `#![recursion_limit = "256"]` is on the working branch, add that line at the top of `src/lib.rs` and `src/main.rs` LOCALLY for builds only; NEVER commit it (`git diff --cached src/lib.rs src/main.rs` must not show it).

Evidence re-verified against source at `57f0109` (= `1a3c8a5` + docs). Locate code by SYMBOL; line numbers drift.

---

## 1. Coverage matrix

Key: rd = readiness.md; a/b/c/d/e/f/g/i/j/l/m/n = raw reports; DI = deferred_items.md. "Blockers" = PLAN-blockers.md, "Product" = PLAN-product.md.

### 1.1 Blockers and their claims
| Finding | Source | Unit |
|---|---|---|
| B1 SSR bin + wasm lib overflow recursion_limit; CI never codegens | rd B1, l, f F1, m | T0 |
| B2 E2E false-green (cargo leptos end-to-end exits 0 on build failure) | rd B2, m, l | T0 |
| B3 leptos_i18n disposed-signal panic kills server (panic=abort) | rd B3, n, f | T1 (+ F-UP upstream issue) |
| B3 secondary: `subscriber_traits.rs:112` variant pinned by inference only | n | T1 (unwind confines every task panic; no separate fix needed — gate: T1 stress) |
| B4 forged `pending_phone` registration; invite redemption unthrottled | rd B4, f, g 1b/Q2, d F1 | U1 |
| B5 swap never succeeds / splits loops / no tx / no release re-validate | rd B5, i, a Gap 1 | U4 |
| B6 TEST_MODE = OTP 000000, no boot guard | rd B6, d F2, g Q3 | U2 |
| B7 no first-admin path | rd B7, d F5, g Q4 | U3 |
| `attr:aria-busy` leak ×13 | rd Major, f, e#44 F1 | A1 |
| Advance Assignment→Delivery unguarded server-side / 0 assignments | rd Minor, a 4.7 | U4 (re-validate on leaving Assignment) |
| `register_with_code` discards UPDATE/commit results (`let _ =`) | g Q2 | U1 (BANNED `let _ =` on registration writes) |
| README `CSRF_SECRET` row; SAMETE_* rows missing | rd Major, d F7 | U2 (U2.7) |
| CI logs 07-03/07-04 expired (410), onset unprovable | m, l | §2 decision D-F13 (unrecoverable; T0 prevents recurrence) |

### 1.2 Product (readiness Majors/Minors already planned)
| Finding | Source | Unit |
|---|---|---|
| Branch not editable after onboarding | rd, a Gap 2 | P-H3 |
| Receipt note + non-receiver identity invisible | rd, a Gap 3, j O1 | P-R |
| Non-enrolled confirm CTA; non-participants told "assigning" | rd, a Gap 4, j P3 | P-H1 |
| Recipient card vanishes on own receipt | rd, j P1 | P-H2 |
| "Не отримав" irreversible / offered day 0 | rd, j P2 | P-H2 (reversible) + §2 D-F1 (no time gate) |
| No meetup info | rd, j P10, a 2.1 | P-M1, P-M2 |
| No site link in SMS | rd, j P11 | P-S |
| Forwarding computed mentally | rd, j O2 | P-F |
| Vacuous E2E T:184/390/398/417/503/649/833/885 | rd, a Gap 6 | P-V, P-H3, P-M2; T:649 → U4 |
| Copy: pen-pal, "кілька хвилин", phantom pages, shipping guidance, not-received next step | rd, j P6–P9, O7 | P-COPY |

### 1.3 Previously unplanned → F-units
| Finding | Source | Unit |
|---|---|---|
| SMS-pumping: unauthenticated `request_otp`, phone-only limits, no IP / global cap | rd Major, d F3, DI "IP-based OTP rate-limiting", g | F-AUTH (commit 1) |
| Lockout after 10 failed verifies/phone/hour MISSING; global SMS cap MISSING | a 1.2-ARCH, Gap 5 | F-AUTH (commit 1) |
| OTP rate-limit counts `otp_codes` rows, which verify deletes → undercount | source `auth::check_otp_rate_limit` (verified) | F-AUTH (commit 1: counts from append-only `auth_events`) |
| OTP attempt increment read-then-write (non-atomic) | g Q3 | F-AUTH (commit 1) |
| Dead duplicate `auth::verify_otp` (no callers) | source grep (verified: 0 callers) | F-AUTH (commit 1) |
| Unit tests for OTP limits absent (dev-protocol promise) | e#53, a 1.2 | F-AUTH (pure decision fns, TDD) + DOC-GUIDE |
| Rate-limited `request_otp` silently "succeeds" (user never told) | g Q3 | F-AUTH (commit 2: explicit error) |
| `request_otp` NewAccount/AccountExists = phone-existence oracle | rd Major, d F4, g | F-AUTH (commit 2) |
| `register_with_code` failure paths redirect with no error | a 1.1 | F-AUTH (commit 2) |
| Legal name: no negative test | a 1.1 | F-AUTH (commit 2 E2E) |
| Dockerfile never built in CI; slim image lacks pkg-config/libssl; unpinned cargo-chef/cargo-leptos | rd Major, d F6, d table | F-DOCKER |
| `cargo chef cook` uses default features (cache miss) | d | F-DOCKER |
| No pre-compression in image | rd Minor, d F10 | F-DOCKER |
| `LEPTOS_ENV` unset (Env::DEV in prod) | rd Minor, d F9 | F-DOCKER |
| Builder base `rust:1.91-slim` (Debian trixie default tags) vs distroless debian12 glibc | source Dockerfile (verified) | F-DOCKER (`-slim-bookworm`) |
| cargo-leptos version drift (0.3.11 vs 0.3.7) | f Setup, e#28 | F-DOCKER (pin 0.3.7 in Dockerfile + CI) + DOC-README |
| CI runs bare `cargo test`; 7 ssr tests never in CI | rd Major, b F2, e F2 | F-CI |
| cargo-audit freshness (last run 07-25); no way to run CI without a push | rd Minor, c, b | F-CI (weekly schedule + `workflow_dispatch`) |
| HEAD has no CI run (skip token); empty-commit workaround | c | F-CI (`workflow_dispatch`) |
| Container/CI audit parity (`cargo audit` only in CI) | b | F-CI (`just audit`, in `just check`) |
| future-incompat warning `proc-macro-error2 v2.0.1` | b INFO | F-CI (weekly job prints `cargo report future-incompatibilities`) + §2 D-F10 |
| Submit hydration gate completeness unverified (23 sites, 1 sampled) | e#43 | F-CI (`scripts/check-ui-invariants.sh`) |
| `attr:` regression has no static guard | e#44, conventions | F-CI (same script) |
| Expired sessions/OTP never swept | rd Minor, d F10 | F-SRV |
| No security headers | d F10 | F-SRV |
| Dry-run logs OTP codes, boot logs nothing about mode | d F8 | U2 (test-mode guard) + F-SRV (WARN banner in dry-run) |
| OTP hash unsalted SHA-256 | d F10, g Q3 | §2 D-F2 (salt cannot protect a 10^6 space) + F-SRV sweep shortens exposure |
| Healthcheck / backups / Coolify DB provisioning undocumented | d F10, d Run/Deploy | DOC-README (deploy checklist) + §2 D-F9 |
| Cancel available pre-launch (Story 4.3 CONTR) | rd Minor, a 4.3 | F-ADV |
| ARCH:518 advance disabled until deadline — not implemented | a 4.7 | F-ADV |
| Generation DELETE precedes N≥3 guard, no tx | rd Minor, a Gap 8 | F-GEN |
| Deactivated users in generation pool | rd Minor, a Gap 8 | F-GEN |
| Deactivated users counted/targeted by nudges and assignment SMS | rd Minor, a Gap 8 | F-SMSF |
| Assignment SMS server-allowed in Assignment phase (pre-release) | a 2.3 | F-SMSF |
| No self/admin deactivate guard | rd Minor, a 6.1 | F-DEACT |
| Organizer not alerted on not-received (Story 2.4 "triggers organizer notification") | rd Major, j O3/C9 | F-ALERT |
| Season dates/theme not editable (deadline extension) | rd Major, j O6/C8 | F-EDIT |
| Theme hidden after enroll | rd Minor, j P5/C11, a 4.1 | F-HOME |
| Enroll form shown after signup deadline | rd Minor, j P4/C12, a 2.1 | F-HOME |
| No countdown (Story 2.2 AC, ARCH admin "Deadline countdown") | rd Minor, j P13/C18, a 2.2 | F-HOME (participant) + F-ADMCD (admin) |
| Guideline copy omits prohibitions + ownership/consent rules | a Gap 7, a 2.1 | F-HOME |
| Newcomer NoSeason has no "how it works" | j P12/C14 | F-HOME |
| Launch/advance → organizer forgets the separate SMS click | j O4/C13 | F-COPY (toasts name the next SMS action) |
| Organizer has no time-remaining on confirm deadline | j O5/C15 | F-ADMCD |
| No per-participant failure history for deactivation decisions | j O9/C17 | F-ADMCD (failed-send count column) |
| Known groups only via raw SQL | j O8/C16 | §2 D-F6 + DOC-README (organizer SQL recipes) |
| Logout not shown on `/onboarding` (Story 1.4 "every authenticated page") | a 1.4 | F-NAV |
| Geometric visual assertions unbuilt | DI, conventions 06-25 | F-GEOM |
| `cohort-seed.sql` bare `ON CONFLICT DO NOTHING` ×5 | DI | F-SEED |
| Coverage gaps a-scope flagged as weak: distributor shown (1.5), used-code non-revocable (1.6), filter (1.6), old session replay after logout (1.4), create-season negative validation (4.1), receipt-nudge count (4.4), participant blocked from /admin (4.3 admin-only) | a trace rows | F-COV |
| `#[cfg(any(feature="ssr", test))]` gate has no WHY (12 sites now, not 10) | DI, d | DOC-OPUS / F-CFG |
| Manifesto SubagentStart hook injects 0 elements (no `other:` key, typo `simlpe-made-easy`, names without sources) | DI, conventions 07-03/07-12 | DOC-OPUS / F-MANI |
| Orphan DB `samete_ssr_debug2` | DI | F-OPS (orchestrator) |
| `implementation_plan.md` decision — file does not exist | DI, d F12, e#50 | F-MEM (delete entry) |
| DI "create-form unreachable after cancel" — stale (form exists, T:801 passes) | rd §2, a 4.5 | F-MEM (delete entry) |
| DI panic entry misattributed ("tower_http 500s") | rd §2, n | T1.4 (deletes entry) |
| leptos_i18n root defect (`try_get` in `context.rs:213`) upstream | n | F-UP |

### 1.4 Doc / claim drift (all DOC-OPUS except memory files)
| Finding | Source | Unit |
|---|---|---|
| CLAUDE.md `just test` 55/62 (actual 68/75; changes again with every unit) | rd §2, b, e#1 | F-DOC-CLAUDE |
| CLAUDE.md / e2e README / POM comment navigationTimeout 15s (actual 30_000) | rd §2, e#9 | F-DOC-CLAUDE, F-DOC-GUIDE (POM comment edited by F-COV) |
| CLAUDE.md `just clippy` "(SSR)" (runs SSR + wasm) | e#3 | F-DOC-CLAUDE |
| CLAUDE.md WASM size claims unverified | e#27, rd §7 | F-DOC-CLAUDE |
| CLAUDE.md Leptos MCP / bridge files env-specific | e#18, e#54 | F-DOC-CLAUDE (marked local-only) |
| README blake2/CSRF false; omits subtle, base64, leptos-use, leptos_i18n; uuid `js` imprecise | rd §2, e#22, e#24 | F-DOC-README |
| README tree omits `invite_codes.rs`, components; recipes table omits `capture-isolated`, `fmt`, `fmt-check` | e#5, e#20 | F-DOC-README |
| README dev setup: cargo-leptos pin, Playwright browsers, disk headroom | f Setup | F-DOC-README |
| e2e README `waitForTimeout` ban lacks the sanctioned `paintSettle` exception | rd §2, e#12 | F-DOC-GUIDE |
| frontend-protocol `!important` ban lacks reduced-motion exception | rd §2, e#40 | F-DOC-GUIDE |
| frontend-protocol z-index map lists unused layers 40/50/60 | rd §2, e#41 | F-DOC-GUIDE |
| dev-protocol "What to Test Where" promises OTP/session unit tests | rd §2, e#53 | F-DOC-GUIDE |
| dev-protocol lint snippet omits `priority = -1` | e#25 | F-DOC-GUIDE |
| Product Spec "No self-registration / organizer creates account" | a PS Entry CONTR, coordinator | F-DOC-SPEC |
| Story 4.2 "launch triggers SMS", 5.4 "~1 hour before", 5.2 timing, ARCH "Release assignments … triggers SMS" vs ARCH:37 manual | a 4.2/5.4, rd §2 | F-DOC-SPEC (manual is canonical, §2 D-F7) |
| ARCH:161 "deactivated rejected at OTP request stage" vs code | a 1.1 | F-DOC-SPEC (doc follows code, which F-AUTH makes uniform) |
| ARCH rate-limit table vs implementation | a 1.2-ARCH | F-DOC-SPEC (matches F-AUTH constants) |
| Story 2.4 "prompt 5 days after assignment" vs organizer-driven nudge | a 2.4 | F-DOC-SPEC |
| codebase_state: "SHIPPED, CI GREEN", 117/119, 119 tests, "all submit buttons aria-busy", module inventory omits components | rd §2, e#6/#7/#19 | F-MEM (orchestrator hand-written; memory work per conventions 07-13) |

### 1.5 Coverage status (not defects)
a-scope rows marked IMPLEMENTED-UNTESTED that are not listed above are behaviour verified by code trace with no defect; they stay as coverage status. The behaviour-critical ones get tests in F-COV/F-AUTH; the rest (e.g. 1.1 "codes never expire" — by spec; 6.1 re-entry via UNIQUE phone) have no failure mode a test would catch beyond what schema constraints already enforce.

Scope-creep list (a) = features beyond spec, not defects. No unit.

## 2. Decisions taken without a competent default in the reports
| ID | Decision | Rationale |
|---|---|---|
| D-F1 | Receipt form stays available from the start of delivery; no time gate. | Same-city next-day delivery is standard: day-1 recipients must be able to confirm. P-H2 makes "not received" reversible; that removes the harm. The ~day-5 prompt is the organizer's receipt-nudge SMS. |
| D-F2 | OTP hash stays unsalted SHA-256. | A salt is stored next to the hash; a 6-digit space is brute-forced in milliseconds either way. Real mitigations: 10-min TTL, single use, F-SRV hourly sweep. A keyed hash would need a secret, which U1 rejected. |
| D-F3 | Rate limits: phone 1/60 s and 5/h (existing); per client IP 10 OTP/h; global 50 OTP SMS/h (ARCH ~50); verify lockout after 10 failed codes per phone per hour. All recorded in one append-only `auth_events` table. | ARCH table values. One log, every limit is a COUNT, testable as a pure decision over counts. 10/h per IP covers a household/office sharing NAT at club scale (≤50 people). |
| D-F4 | Client IP source is explicit config: `SAMETE_CLIENT_IP_SOURCE=peer` (default) or `x-forwarded-for` (prod behind Coolify/Traefik: take the LAST hop, appended by the trusted proxy). | Trusting XFF without a proxy lets attackers rotate IPs; the last hop is the only proxy-written value. |
| D-F5 | Invite codes keep 2-word entropy and never expire. | Story 1.1 AC "codes never expire". Guessing is capped by U1 (5 tries per OTP ticket) × F-AUTH (5 OTP/h/phone, 10/h/IP, 50/h global). |
| D-F6 | No known-groups admin UI. README gets copy-paste SQL recipes. | Product Spec §App Scope: "organizer edits database directly". |
| D-F7 | SMS stay organizer-triggered. Stories and ARCH rows that imply automatic SMS are rewritten. | ARCH:37 deliberate "no cron, no scheduler". F-COPY toasts point the organizer at the next SMS action. |
| D-F8 | Cancel is offered only for launched seasons (Story 4.3). Mistakes in an unlaunched season are fixed with F-EDIT. | Story is the contract. Edit covers the pre-launch correction need. |
| D-F9 | No Docker HEALTHCHECK (distroless has no shell/curl). Coolify HTTP health check on `/login` is documented. | Only workable probe for a distroless image. |
| D-F10 | `proc-macro-error2` future-incompat stays (transitive via leptos macros). The weekly CI job prints the report. | No repo-side fix exists; visibility is the fix. |
| D-F11 | Advance gates: Enrollment→Preparation needs `signup_deadline` passed; Preparation→Assignment needs `confirm_deadline` passed; bypassed in test mode. | ARCH:518; mirrors participant-side deadline gates. Without it the organizer could close confirmation early. |
| D-F12 | Season edit: theme editable in any non-terminal phase. Signup deadline editable only in Enrollment while still in the future. Confirm deadline editable in Enrollment/Preparation. New deadlines must be in the future and signup < confirm. | A deadline that already passed cannot be un-passed (participants already acted on it). |
| D-F13 | Expired CI logs: no action. | Unrecoverable; T0's guard prevents recurrence. |
| D-F14 | Organizer not-received alert SMS carries no link. | Organizer knows the admin URL. Keeps F-ALERT independent of P-S's `Config.site_url`. |
| D-F15 | Countdown = server-computed "N днів" label, no ticking timer. | Same string on SSR and WASM (no hydration mismatch). Day granularity matches 5–14-day phases. |
| D-F16 | Deactivation reason not stored. | F-ADMCD's failure count supplies the evidence the reason would record. YAGNI. |
| D-F17 | Dry-run in production is not refused (only test mode is, U2). Boot logs a WARN banner. | Dry-run is a legitimate staging mode. The banner makes a misconfiguration visible in logs. |

---

## 3. Lanes

| Lane | Base | Units / commits (in order inside the lane) | Write-set | Symbols from other units (contract, never a wait) |
|---|---|---|---|---|
| F-AUTH | default | 1 `F-AUTH-RL` · 2 `F-AUTH-ORACLE` | `migrations/20261004000003_auth_events.sql` (new), `src/auth.rs`, `src/config.rs`, `src/main.rs`, `src/pages/login.rs`, `locales/uk.json`, `README.md`, `.sqlx/*`, `end2end/tests/mail_club.spec.ts`, POM | U2's `test_mode` plumbing: `check_otp_send_allowed(…, test_mode: bool)` takes a bool; call site on base computes it the way the base does (env read); after U2 merges the integrator passes `config.test_mode()`. U1 edits `verify_otp_code`'s branches; F-AUTH changes only the OTP-check call inside it; the integrator keeps U1's branches. |
| F-SRV | default | `F-SRV` | `src/db.rs`, `src/main.rs`, `Cargo.toml` (tower-http feature) | none |
| F-DOCKER | default | `F-DOCKER` | `Dockerfile`, `.github/workflows/ci.yml` (new `docker` job only) | T0 recursion fix needed for the image build to succeed. The Dockerfile is written now; the docker job's green run proves it once T0 is on the branch under test. Contract: T0.1/T0.2 exact lines. |
| F-CI | default | `F-CI` | `.github/workflows/ci.yml` (Check job + triggers), `justfile`, `scripts/check-ui-invariants.sh` (new) | A1 removes the 13 `attr:aria-busy` sites. Until A1 merges, the invariant script fails on exactly those 13 lines (gate below). |
| F-GEN | default + `git merge --no-edit worktree-agent-a46c4c68440f8d984` | `F-GEN` | `src/admin/assignments.rs`, `.sqlx/*` | U4 (on that branch) rewrote `store_and_build_preview`'s `AssignmentLink` literal; F-GEN changes that fn's signature, so it starts from the existing U4 text |
| F-SMSF | default | `F-SMSF` | `src/admin/sms.rs`, `locales/uk.json` (delete one key), `.sqlx/*` | none |
| F-DEACT | default | `F-DEACT` | `src/admin/participants.rs`, `locales/uk.json`, `.sqlx/*` | none |
| F-ADV | default + `git merge --no-edit worktree-agent-a46c4c68440f8d984` | `F-ADV` | `src/admin/season.rs`, `src/admin/state.rs`, `src/admin/page.rs`, `locales/uk.json`, `.sqlx/*`, spec, POM | U4's release guard sits in `advance_season`; F-ADV adds its guard in the same fn, so it starts from that existing text |
| F-EDIT | default | `F-EDIT` | `src/admin/season.rs`, `src/admin/state.rs`, `src/admin/page.rs`, `locales/uk.json`, `.sqlx/*`, spec, POM, `capture-constants.ts` | P-V `formatDateUk`: copy verbatim from PLAN-product §V.1 if absent (integration keeps one) |
| F-ALERT | default | `F-ALERT` | `src/admin/sms.rs` (helper only), `src/pages/home.rs` (`confirm_receipt` call site), `locales/uk.json`, `.sqlx/*` | P-H2 rewrites `confirm_receipt`'s UPDATE; F-ALERT adds 4 lines after the successful update. Integrator re-places them after P-H2's guarded update (`affected == 1` and `new_status == NotReceived`) |
| F-HOME | default | `F-HOME` | `src/pages/home.rs`, `src/date_format.rs`, `src/lib.rs` (cfg move), `locales/uk.json`, `.sqlx/*`, spec, POM, `end2end/tests/visual-audit.spec.ts` | P-H1/P-H2/P-H3/P-M2 also reshape `HomeState`; each change is additive (new fields/variants). The integrator unions them; the exhaustive matches force every arm to be handled |
| F-ADMCD | default | `F-ADMCD` | `src/date_format.rs`, `src/lib.rs` (cfg move), `src/admin/state.rs`, `src/admin/participants.rs`, `src/admin/page.rs`, `locales/uk.json`, `.sqlx/*`, spec, POM | `date_format::days_remaining`/`DayCount`: identical block to F-HOME §HOME.1 (copy verbatim; integration keeps one) |
| F-COPY | default | `F-COPY` | `locales/uk.json` (existing values only) | none |
| F-NAV | default | `F-NAV` | `src/app.rs`, spec, POM | none |
| F-GEOM | default | `F-GEOM` | `end2end/tests/fixtures/geometry.ts` (new), `end2end/tests/visual-audit.spec.ts`, `style/components.css` (only if a real overflow is found) | none |
| F-SEED | default | `F-SEED` | `end2end/tests/fixtures/cohort-seed.sql` | none |
| F-COV | default | `F-COV` | spec, POM | none (all testids exist at base) |
| DOC-OPUS (opus) | default | `F-DOC-README` · `F-DOC-CLAUDE` · `F-DOC-GUIDE` · `F-DOC-SPEC` · `F-MANI` · `F-CFG` · `F-UP` | `README.md`, `CLAUDE.md`, `guidance/dev-protocol.md`, `guidance/frontend-protocol.md`, `end2end/README.md`, `spec/product/Product Spec.md`, `spec/technical/User Stories.md`, `spec/technical/Architecture.md`, `.manifestos.yaml`, `src/**` (comment lines only, F-CFG), `orchestration_log/history/2026-10-04/plans/upstream-leptos-i18n-issue.md` (new) | Describes behaviour of F-AUTH, F-ADV, F-EDIT, F-DOCKER, F-CI, P-* exactly as specified in this plan and PLAN-product.md (contracts) |
| ORCH (orchestrator, not delegated) | — | `F-OPS` · `F-MEM` | dev Postgres; `orchestration_log/reference/{deferred_items,codebase_state}.md` | none |

"spec" = `end2end/tests/mail_club.spec.ts`; "POM" = `end2end/tests/fixtures/mail_club_page.ts`. Default base = `claude/loving-johnson-7l8hn5`. Migration numbering: U1 `…000001`, P-M1/P-M2 `…000002`, F-AUTH `…000003`.

Integration notes (integrator, not lanes): union `locales/uk.json` keys; after any merge that touches a `query!` run `cargo sqlx prepare --workspace -- --features ssr`; E2E tests sharing an anchor title are ordered as listed in each unit.

## 4. Shared environment (every code lane)

Restates `PLAN-blockers.md` §Shared environment; the container E2E recipe (browser path, tailwind shim) there is binding.

```bash
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0
pg_isready -h localhost -p 5432 || docker compose -f /home/user/same-te-mail-club/docker-compose.yml up -d
export DATABASE_URL=postgres://samete:samete@localhost:5432/samete_<lane>   # never `samete`
sqlx database drop -y; sqlx database create && sqlx migrate run
```
sqlx cache after ANY query text change: `cargo sqlx prepare --workspace -- --features ssr` → exit 0, `.sqlx/` changes committed.

Baseline before editing (paste): `cargo test 2>&1 | grep -E "^test result"` and `SQLX_OFFLINE=true cargo test --features ssr 2>&1 | grep -E "^test result"` → summed passed = `BARE_BASE`, `SSR_BASE`; E2E baseline = one isolated-harness run on the base.

Standard gates:
```bash
cargo fmt --all -- --check
SQLX_OFFLINE=true cargo clippy --features ssr --no-default-features -- -D warnings
SQLX_OFFLINE=true cargo clippy --target wasm32-unknown-unknown --features hydrate --no-default-features -- -D warnings
SQLX_OFFLINE=true cargo build --no-default-features --features ssr --bin samete
SQLX_OFFLINE=true cargo build --lib --target wasm32-unknown-unknown --no-default-features --features hydrate
cargo test 2>&1 | grep -E "^test result"
SQLX_OFFLINE=true cargo test --features ssr 2>&1 | grep -E "^test result"
```
**REQUIRED:** fmt silent, exit 0; clippy/build end `Finished`, zero `warning:`/`error:`; every `test result:` `ok.` `0 failed`; passed = base + unit delta (bare/ssr deltas stated per unit).

E2E gate (only units that list one): `bash scripts/isolated-capture.sh e2e_<lane>_<n> full > /tmp/e2e_<lane>_<n>.log 2>&1; echo "exit=$?" >> /tmp/e2e_<lane>_<n>.log`, run in background, read the FILE. **REQUIRED:** `exit=0`, `N passed` = base + unit E2E delta, no `failed`, no `flaky`, `2 skipped`; every named title passed. 3 consecutive runs.

Commit: one-line conventional message (given per unit), no body, no `Co-Authored-By`, no AI attribution; worktree only; report SHA. Two-failure rule: a gate failing twice → STOP, report BLOCKED with output.

Read hygiene: chunked reads ≤400 lines; never read `target/`, `node_modules/`, screenshots (reviewers excepted).

---

# F-AUTH-RL — OTP abuse limits: per-phone, per-IP, global, verify lockout

## Why This Matters
`request_otp` is public and sends a paid SMS to any Ukrainian number. Limits today are per phone only, and they count `otp_codes` rows, which verification deletes, so they undercount. No IP dimension, no global cap, no lockout. The attempt counter is read-then-write.

## What You Must Do
### RL.1 Migration (exact) `migrations/20261004000003_auth_events.sql`
```sql
-- Append-only log of OTP sends and failed verifications; every auth rate limit
-- is a COUNT over a time window of this table. Rows older than 1 day are pruned
-- on insert (record_auth_event).
CREATE TYPE auth_event_kind AS ENUM ('otp_sent', 'otp_verify_failed');

CREATE TABLE auth_events (
    id BIGSERIAL PRIMARY KEY,
    kind auth_event_kind NOT NULL,
    phone TEXT NOT NULL,
    client_ip TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_auth_events_kind_created ON auth_events (kind, created_at DESC);
CREATE INDEX idx_auth_events_phone_kind_created ON auth_events (phone, kind, created_at DESC);
CREATE INDEX idx_auth_events_ip_kind_created ON auth_events (client_ip, kind, created_at DESC);
```
Gate: `ls migrations | sort | tail -1` → this file (if a later-numbered file exists on base → BLOCKED).

### RL.2 TDD — pure decisions in `src/auth.rs`
Step 1 — add to `mod tests` (red):
```rust
    use super::{
        ClientIpSource, OtpSendCounts, OtpSendDenied, otp_send_allowed, resolve_client_ip,
        verification_locked,
    };
    use std::net::IpAddr;

    fn counts(m: i64, h: i64, ip: i64, g: i64) -> OtpSendCounts {
        OtpSendCounts { phone_last_minute: m, phone_last_hour: h, ip_last_hour: ip, global_last_hour: g }
    }

    #[test]
    fn fresh_phone_may_receive_otp() {
        assert_eq!(otp_send_allowed(counts(0, 0, 0, 0)), Ok(()));
    }

    #[test]
    fn phone_cooldown_blocks_second_sms_within_a_minute() {
        assert_eq!(otp_send_allowed(counts(1, 1, 1, 1)), Err(OtpSendDenied::PhoneCooldown));
    }

    #[test]
    fn phone_hourly_cap_is_five() {
        assert_eq!(otp_send_allowed(counts(0, 4, 4, 4)), Ok(()));
        assert_eq!(otp_send_allowed(counts(0, 5, 5, 5)), Err(OtpSendDenied::PhoneHourly));
    }

    #[test]
    fn ip_hourly_cap_is_ten() {
        assert_eq!(otp_send_allowed(counts(0, 0, 9, 9)), Ok(()));
        assert_eq!(otp_send_allowed(counts(0, 0, 10, 10)), Err(OtpSendDenied::IpHourly));
    }

    #[test]
    fn global_cap_is_fifty_and_checked_first() {
        assert_eq!(otp_send_allowed(counts(1, 5, 10, 50)), Err(OtpSendDenied::GlobalHourly));
        assert_eq!(otp_send_allowed(counts(0, 0, 0, 49)), Ok(()));
    }

    #[test]
    fn ten_failed_verifications_lock_the_phone() {
        assert!(!verification_locked(9));
        assert!(verification_locked(10));
    }

    #[test]
    fn peer_source_ignores_forwarded_header() {
        let peer: IpAddr = "10.0.0.7".parse().expect("ip");
        assert_eq!(resolve_client_ip(ClientIpSource::Peer, Some("1.2.3.4"), peer), peer);
    }

    #[test]
    fn forwarded_source_takes_last_hop() {
        let peer: IpAddr = "10.0.0.7".parse().expect("ip");
        let last: IpAddr = "5.6.7.8".parse().expect("ip");
        assert_eq!(resolve_client_ip(ClientIpSource::LastForwardedFor, Some("1.2.3.4, 5.6.7.8"), peer), last);
    }

    #[test]
    fn forwarded_source_falls_back_to_peer_on_missing_or_garbage() {
        let peer: IpAddr = "10.0.0.7".parse().expect("ip");
        assert_eq!(resolve_client_ip(ClientIpSource::LastForwardedFor, None, peer), peer);
        assert_eq!(resolve_client_ip(ClientIpSource::LastForwardedFor, Some("nonsense"), peer), peer);
    }
```
Step 2 — `SQLX_OFFLINE=true cargo test --features ssr auth::tests` → red. Step 3 — implement in `src/auth.rs`:
```rust
/// OTP SMS per phone per rolling hour.
pub const OTP_PER_PHONE_PER_HOUR: i64 = 5;
/// OTP SMS per client IP per rolling hour.
pub const OTP_PER_IP_PER_HOUR: i64 = 10;
/// OTP SMS for the whole server per rolling hour (SMS-pumping circuit breaker).
pub const OTP_GLOBAL_PER_HOUR: i64 = 50;
/// Failed code entries per phone per rolling hour before verification locks.
pub const VERIFY_FAILURES_PER_PHONE_PER_HOUR: i64 = 10;

/// Recent `otp_sent` counts that decide whether another OTP may be sent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OtpSendCounts {
    pub phone_last_minute: i64,
    pub phone_last_hour: i64,
    pub ip_last_hour: i64,
    pub global_last_hour: i64,
}

/// Which limit refused an OTP send.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OtpSendDenied {
    GlobalHourly,
    IpHourly,
    PhoneCooldown,
    PhoneHourly,
}

/// Rate-limit decision for one OTP send (global first: it protects the budget).
///
/// # Errors
///
/// Returns the first limit the counts reach.
pub fn otp_send_allowed(c: OtpSendCounts) -> Result<(), OtpSendDenied> {
    if c.global_last_hour >= OTP_GLOBAL_PER_HOUR {
        return Err(OtpSendDenied::GlobalHourly);
    }
    if c.ip_last_hour >= OTP_PER_IP_PER_HOUR {
        return Err(OtpSendDenied::IpHourly);
    }
    if c.phone_last_minute >= 1 {
        return Err(OtpSendDenied::PhoneCooldown);
    }
    if c.phone_last_hour >= OTP_PER_PHONE_PER_HOUR {
        return Err(OtpSendDenied::PhoneHourly);
    }
    Ok(())
}

/// Verification is locked for a phone once it reaches the hourly failure cap.
pub fn verification_locked(failures_last_hour: i64) -> bool {
    failures_last_hour >= VERIFY_FAILURES_PER_PHONE_PER_HOUR
}

/// Where the client IP comes from (`SAMETE_CLIENT_IP_SOURCE`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientIpSource {
    /// TCP peer address (no proxy, or proxy not trusted).
    Peer,
    /// Last `X-Forwarded-For` hop — the value appended by the one trusted
    /// reverse proxy (Coolify/Traefik). Earlier hops are client-controlled.
    LastForwardedFor,
}

/// Resolve the client IP for rate limiting. Falls back to the peer when the
/// header is missing or unparsable.
pub fn resolve_client_ip(source: ClientIpSource, forwarded_for: Option<&str>, peer: std::net::IpAddr) -> std::net::IpAddr {
    match source {
        ClientIpSource::Peer => peer,
        ClientIpSource::LastForwardedFor => forwarded_for
            .and_then(|v| v.rsplit(',').next())
            .and_then(|hop| hop.trim().parse().ok())
            .unwrap_or(peer),
    }
}
```
Step 4 — green (9 new, ssr only: `auth` is ssr-gated).

### RL.3 DB layer (`src/auth.rs`)
```rust
/// Kinds of rows in `auth_events`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "auth_event_kind", rename_all = "snake_case")]
pub enum AuthEventKind {
    OtpSent,
    OtpVerifyFailed,
}

/// Append one auth event and prune events older than a day (bounded table).
///
/// # Errors
///
/// Returns `Err` on database failure.
pub async fn record_auth_event(pool: &PgPool, kind: AuthEventKind, phone: &str, client_ip: &str) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    sqlx::query!("DELETE FROM auth_events WHERE created_at < now() - interval '1 day'")
        .execute(&mut *tx)
        .await?;
    sqlx::query!(
        r#"INSERT INTO auth_events (kind, phone, client_ip) VALUES ($1, $2, $3)"#,
        kind as AuthEventKind,
        phone,
        client_ip,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

/// Refuse an OTP send that would exceed any limit (skipped in test mode).
///
/// # Errors
///
/// `Err(AppError::RateLimited)` when a limit is reached; `Err(AppError::Database)` on DB failure.
pub async fn check_otp_send_allowed(pool: &PgPool, phone: &str, client_ip: &str, test_mode: bool) -> Result<(), AppError> {
    if test_mode {
        return Ok(());
    }
    let row = sqlx::query!(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE phone = $1 AND created_at > now() - interval '60 seconds') AS "phone_last_minute!",
            COUNT(*) FILTER (WHERE phone = $1) AS "phone_last_hour!",
            COUNT(*) FILTER (WHERE client_ip = $2) AS "ip_last_hour!",
            COUNT(*) AS "global_last_hour!"
        FROM auth_events
        WHERE kind = 'otp_sent' AND created_at > now() - interval '1 hour'
        "#,
        phone,
        client_ip,
    )
    .fetch_one(pool)
    .await?;
    otp_send_allowed(OtpSendCounts {
        phone_last_minute: row.phone_last_minute,
        phone_last_hour: row.phone_last_hour,
        ip_last_hour: row.ip_last_hour,
        global_last_hour: row.global_last_hour,
    })
    .map_err(|denied| {
        tracing::warn!(?denied, phone, client_ip, "OTP send refused by rate limit");
        AppError::RateLimited
    })
}

/// Why an OTP verification failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OtpVerifyError {
    /// Phone reached the hourly failure cap.
    Locked,
    /// No live code, attempts exhausted, or wrong code.
    Rejected,
}

/// Verify `code` for `phone` atomically: one attempt is consumed BEFORE the
/// compare (`UPDATE … attempts < 3 RETURNING`), so concurrent guesses cannot
/// exceed 3 per code. Success deletes the code. Failures are logged as events.
///
/// # Errors
///
/// `Err(Locked)` past the failure cap, `Err(Rejected)` otherwise; DB errors map to `Rejected`
/// after logging.
pub async fn verify_otp_for_phone(pool: &PgPool, phone: &str, code: &str, client_ip: &str) -> Result<(), OtpVerifyError> {
    let failures = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) AS "count!"
        FROM auth_events
        WHERE kind = 'otp_verify_failed' AND phone = $1
          AND created_at > now() - interval '1 hour'
        "#,
        phone,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "verify: failure count query failed");
        OtpVerifyError::Rejected
    })?;
    if verification_locked(failures) {
        return Err(OtpVerifyError::Locked);
    }

    let row = sqlx::query!(
        r#"
        UPDATE otp_codes SET attempts = attempts + 1
        WHERE id = (
            SELECT id FROM otp_codes
            WHERE phone = $1 AND expires_at > now()
            ORDER BY created_at DESC
            LIMIT 1
        )
          AND attempts < 3
        RETURNING id, code_hash
        "#,
        phone,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "verify: attempt update failed");
        OtpVerifyError::Rejected
    })?;

    let matched = row
        .as_ref()
        .is_some_and(|r| constant_time_hash_eq(&sha256_hex(code), &r.code_hash));
    if !matched {
        if let Err(e) = record_auth_event(pool, AuthEventKind::OtpVerifyFailed, phone, client_ip).await {
            tracing::error!(error = %e, "verify: failed to record failure event");
        }
        return Err(OtpVerifyError::Rejected);
    }
    if let Some(r) = row {
        sqlx::query!("DELETE FROM otp_codes WHERE id = $1", r.id)
            .execute(pool)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "verify: code delete failed");
                OtpVerifyError::Rejected
            })?;
    }
    Ok(())
}
```
Delete: `auth::check_otp_rate_limit`, `auth::verify_otp` (dead: `grep -rn "verify_otp(" src` has no caller), its `OtpRow` if then unused, and `pages/login.rs::verify_otp_for_phone`.

### RL.4 Config (`src/config.rs`)
Add field `pub client_ip_source: crate::auth::ClientIpSource,` and error variant:
```rust
    #[error("SAMETE_CLIENT_IP_SOURCE must be `peer` or `x-forwarded-for`, got {0}")]
    InvalidClientIpSource(String),
```
Pure parser + tests (in config's test module; create `#[cfg(test)] mod tests` if absent):
```rust
/// Parse `SAMETE_CLIENT_IP_SOURCE`; unset = `peer`.
fn client_ip_source_from_var(raw: Option<&str>) -> Result<ClientIpSource, ConfigError> {
    match raw {
        None | Some("peer") => Ok(ClientIpSource::Peer),
        Some("x-forwarded-for") => Ok(ClientIpSource::LastForwardedFor),
        Some(other) => Err(ConfigError::InvalidClientIpSource(other.to_owned())),
    }
}
```
```rust
    #[test]
    fn client_ip_source_defaults_to_peer() {
        assert_eq!(client_ip_source_from_var(None).ok(), Some(ClientIpSource::Peer));
    }

    #[test]
    fn client_ip_source_accepts_forwarded_for() {
        assert_eq!(client_ip_source_from_var(Some("x-forwarded-for")).ok(), Some(ClientIpSource::LastForwardedFor));
    }

    #[test]
    fn client_ip_source_rejects_unknown() {
        assert!(matches!(client_ip_source_from_var(Some("X-Real-IP")), Err(ConfigError::InvalidClientIpSource(_))));
    }
```
`from_env`: `let client_ip_source = client_ip_source_from_var(std::env::var("SAMETE_CLIENT_IP_SOURCE").ok().as_deref())?;`. (Integrator: U2/U3/P-S add other fields; union them.)

### RL.5 Wiring
- `src/main.rs`: `axum::serve(listener, app.into_make_service())` → `axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>())`.
- `src/pages/login.rs`: add
```rust
/// Client IP for rate limiting (see `auth::resolve_client_ip`).
#[cfg(feature = "ssr")]
async fn client_ip(config: &crate::config::Config) -> Result<String, ServerFnError> {
    let peer = leptos_axum::extract::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .await
        .map_err(|e| ServerFnError::new(format!("no peer address: {e}")))?
        .0
        .ip();
    let forwarded_for = leptos::context::use_context::<http::request::Parts>()
        .and_then(|p| p.headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()).map(str::to_owned));
    Ok(crate::auth::resolve_client_ip(config.client_ip_source, forwarded_for.as_deref(), peer).to_string())
}
```
- `request_otp`: compute `let ip = client_ip(&config).await?;` after normalization; replace the `check_otp_rate_limit` block with `auth::check_otp_send_allowed(&pool, &normalized, &ip, <test_mode as on base>)` (the commit-2 error mapping replaces the silent return); after `send_sms` succeeds: `auth::record_auth_event(&pool, auth::AuthEventKind::OtpSent, &normalized, &ip).await.map_err(|e| ServerFnError::new(e.to_string()))?;`.
- `verify_otp_code`: `let ip = client_ip(&config).await?;` (get `config` from context as `request_otp` does); replace `verify_otp_for_phone(&pool, &normalized, &code).await` with `auth::verify_otp_for_phone(&pool, &normalized, &code, &ip).await`. `Err(OtpVerifyError::Locked)` and `Err(Rejected)` both redirect to `/login?otp_error=1` (no lock oracle).
- `README.md` env table, add after the last `SAMETE_*` row:
```
| `SAMETE_CLIENT_IP_SOURCE` | No | `peer` (default) or `x-forwarded-for` — set `x-forwarded-for` in production behind Coolify/Traefik so per-IP OTP limits see the real client (last X-Forwarded-For hop) |
```
- sqlx prepare.

## Verification Gates
```bash
grep -rn "check_otp_rate_limit\|fn verify_otp(\|fn verify_otp_for_phone" src/
```
**REQUIRED:** exactly one match: `src/auth.rs: pub async fn verify_otp_for_phone`.
Live limit probe (release build, no test mode, own port + DB):
```bash
export RLDB=postgres://samete:samete@localhost:5432/samete_frl
DATABASE_URL=$RLDB sqlx database drop -y; DATABASE_URL=$RLDB sqlx database create; DATABASE_URL=$RLDB sqlx migrate run
cargo leptos build --release > /tmp/frl-build.log 2>&1; echo "build=$?"
LEPTOS_SITE_ADDR=127.0.0.1:3981 LEPTOS_SITE_ROOT=target/site LEPTOS_SITE_PKG_DIR=pkg LEPTOS_OUTPUT_NAME=samete \
  DATABASE_URL=$RLDB SAMETE_SMS_DRY_RUN=true ./target/release/samete > /tmp/frl-srv.log 2>&1 &
SRV=$!; until curl -sf -o /dev/null http://127.0.0.1:3981/login; do sleep 0.5; done
ACTION=$(curl -s http://127.0.0.1:3981/login | grep -oE 'action="/api/request_otp[0-9]*"' | head -1 | cut -d'"' -f2)
post() { curl -s -X POST "http://127.0.0.1:3981$ACTION" --data-urlencode "phone=$1"; }
post +380671110000 > /tmp/frl-1.txt; post +380671110000 > /tmp/frl-2.txt
for i in 1 2 3 4 5 6 7 8 9; do post +38067111000$i > /dev/null; done; post +380671119999 > /tmp/frl-ip.txt
psql $RLDB -tAc "select count(*) from auth_events where kind='otp_sent'"
kill $SRV; DATABASE_URL=$RLDB sqlx database drop -y
grep -c "Забагато" /tmp/frl-1.txt /tmp/frl-2.txt /tmp/frl-ip.txt
```
**REQUIRED:** `build=0`; psql count `10`; grep counts `0`, `1`, `1` (2nd same-phone request in 60 s refused; 11th distinct phone from one IP refused). If `ACTION` is empty, paste the `/login` form markup and report BLOCKED.
Standard gates: bare +0, ssr +12. E2E ×3, delta 0 (proves ConnectInfo extraction works under the harness; test mode bypasses the counts).

Commit: `fix(auth): add per-IP, global and lockout OTP limits over an auth event log`

---

# F-AUTH-ORACLE — Uniform `request_otp`, visible registration errors

## Why This Matters
`request_otp` returns `NewAccount`/`AccountExists` to any caller, which makes it an account-existence oracle; the client treats both identically (`login.rs` `otp_step` Memo). When rate-limited it pretends success. `register_with_code` failure paths redirect with no message.

## What You Must Do
Keys (append to `locales/uk.json`, F-AUTH owns):
```json
"login_rate_limited": "Забагато запитів коду. Спробуй пізніше.",
"auth_register_failed": "Не вдалося створити акаунт. Перевір код та ім'я і спробуй ще раз."
```
1. Delete `enum RequestOtpOutcome`. `request_otp` signature → `pub async fn request_otp(phone: String) -> Result<(), ServerFnError>`. Every `return Ok(RequestOtpOutcome::…)` → `return Ok(())`. Remove the `user_exists` query entirely. Rate-limit refusal → `return Err(ServerFnError::new(td_string!(Locale::uk, login_rate_limited)));`. Rewrite the fn doc: "Sends an OTP to any well-formed phone. The response is identical for registered, deactivated and unknown phones; routing happens only after verification. # Errors: rate limit (same message for every phone), SMS delivery failure."
2. Client: `otp_step` Memo → `matches!(request_action.value().get(), Some(Ok(())))`. Any other `RequestOtpOutcome` reference removed (`grep` gate).
3. `register_with_code`: every failure branch that today redirects to `/login?pending=1` redirects to `/login?pending=1&register_error=1` (branches: empty name, tx begin, code lookup, code not unused, user insert; plus whatever failure branches U1 adds — integrator applies the same suffix). The branch that redirects to plain `/login` (no ticket/phone) is unchanged.
4. `LoginPage`: add `is_register_error`, the same SSR/client dual read as `is_otp_error`, with `register_error=1`. Render on the invite-code step, directly above its submit button:
```rust
            <Show when=move || is_register_error>
                <p class="field-error" role="alert" aria-live="assertive" data-testid="register-error">
                    {t!(i18n, auth_register_failed)}
                </p>
            </Show>
```
(Thread `is_register_error: bool` into the step component exactly as `is_otp_error` is threaded.)
5. E2E. POM:
```ts
  async submitLegalName(name: string) {
    await expect(this.page.getByTestId("create-account-button")).toBeEnabled();
    await this.page.getByTestId("legal-name-input").fill(name);
    await this.clickAndWaitForResponse(this.page.getByTestId("create-account-button"), "register_with_code");
  }

  async expectRegisterError() {
    await expect(this.page.getByTestId("register-error")).toBeVisible();
  }
```
Spec: `EXTRA_PHONES` add `BLANK_NAME_TEST: "+380670000011",`; `CODES` add `BLANK: ""`. Insert after `"1.1 — used invite code is rejected"`:
```ts
    // Story 1.1 AC: legal name required — whitespace-only is refused with a visible error.
    test("1.1 — blank legal name is rejected with a visible error", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      CODES.BLANK = await app.generateInviteCode();
      await app.reachInviteCodeStep(EXTRA_PHONES.BLANK_NAME_TEST);
      await page.getByTestId("invite-code-input").fill(CODES.BLANK);
      await page.getByTestId("submit-invite-code-button").click();
      await app.submitLegalName("   ");
      await app.expectRegisterError();
      await app.login(ADMIN_PHONE);
      await app.revokeInviteCode(CODES.BLANK); // keeps later "3 active participants" counts intact
    });
```
(The two raw `getByTestId` lines mirror the existing neighbouring tests' shape. If U1's POM `submitInviteCode` is present on the integrated branch, the integrator swaps them for it.)

## Verification Gates
```bash
grep -rn "RequestOtpOutcome\|NewAccount\|AccountExists" src/
grep -c "register_error=1" src/pages/login.rs
```
**REQUIRED:** zero matches; count ≥ `6` (5 redirects + the query read).
Oracle probe (reuse RL probe server setup; seed one user `psql $RLDB -c "insert into users (phone,name) values ('+380671230000','X')"`): `post +380671230000` and `post +380671239999` (different phones, from a fresh DB so no limit) → **REQUIRED:** byte-identical bodies (`cmp` exit 0).
Standard gates (deltas 0). E2E ×3: delta +1; title `1.1 — blank legal name is rejected with a visible error`.

Commit: `fix(auth): make OTP requests uniform and surface registration errors`

---

# F-SRV — Expiry sweep, security headers, dry-run banner

## What You Must Do
1. `src/db.rs`:
```rust
/// Delete expired sessions and OTP codes. Returns rows deleted.
///
/// # Errors
///
/// Returns `Err` on database failure.
pub async fn sweep_expired(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let sessions = sqlx::query!("DELETE FROM sessions WHERE expires_at < now()")
        .execute(pool)
        .await?
        .rows_affected();
    let otps = sqlx::query!("DELETE FROM otp_codes WHERE expires_at < now()")
        .execute(pool)
        .await?
        .rows_affected();
    Ok(sessions + otps)
}
```
2. `src/main.rs`, after migrations:
```rust
    // Hourly sweep: expired sessions/OTP codes are otherwise only removed when touched.
    {
        let pool = pool.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600));
            loop {
                interval.tick().await;
                match samete::db::sweep_expired(&pool).await {
                    Ok(n) => tracing::info!(deleted = n, "expired auth rows swept"),
                    Err(e) => tracing::warn!(error = %e, "expired auth row sweep failed"),
                }
            }
        });
    }
```
and, right after the config is loaded:
```rust
    if config.sms_dry_run {
        tracing::warn!("SAMETE_SMS_DRY_RUN=true: SMS are NOT sent; message bodies (including OTP codes) go to this log");
    }
```
(After U2 merges, the integrator rewrites the condition as `matches!(config.sms, SmsMode::DryRun { .. })`.)
3. Security headers. `Cargo.toml` tower-http features → `["compression-br", "trace", "set-header"]`. In the router chain, before `.layer(CompressionLayer::new())`:
```rust
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::REFERRER_POLICY,
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
```
Imports: `use axum::http::{HeaderValue, header}; use tower_http::set_header::SetResponseHeaderLayer;`. HSTS is set at Traefik (DOC-README).
4. sqlx prepare.

## Verification Gates
Release binary on its own port + DB (launcher as in the F-AUTH probe; port 3982, `SRVDB=postgres://samete:samete@localhost:5432/samete_fsrv`, server log `/tmp/fsrv-srv.log`, `SAMETE_SMS_DRY_RUN=true`). Before starting the server, seed one expired session:
```bash
psql $SRVDB -c "insert into users (id,phone,name) values ('00000000-0000-0000-0000-0000000000aa','+380670009999','S')"
psql $SRVDB -c "insert into sessions (token_hash,user_id,expires_at) values ('x','00000000-0000-0000-0000-0000000000aa', now()-interval '1 day')"
```
Start the server (the interval's first tick fires immediately), wait for `/login`, then:
```bash
curl -sI http://127.0.0.1:3982/login | grep -iE "x-content-type-options: nosniff|x-frame-options: DENY|referrer-policy: strict-origin-when-cross-origin" | wc -l
grep -c "SMS are NOT sent" /tmp/fsrv-srv.log
grep -c "expired auth rows swept" /tmp/fsrv-srv.log
psql $SRVDB -tAc "select count(*) from sessions"
git diff --stat Cargo.lock
```
**REQUIRED:** `3`; `1`; `1`; `0`; Cargo.lock unchanged or changed only in tower-http's feature-resolved entries (no new `[[package]]`: `git diff Cargo.lock | grep -c '^+name = '` → `0`).
Standard gates (deltas 0). E2E ×3, delta 0.

Commit: `feat(server): sweep expired auth rows, add security headers, warn on dry-run`

---

# F-DOCKER — Production image builds in CI

## What You Must Do
1. `Dockerfile` (final content):
```dockerfile
# Builder matches the runtime's Debian release (distroless cc-debian12 = bookworm):
# a binary linked against a newer glibc would not start.
FROM rust:1.97-slim-bookworm AS chef
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev brotli \
    && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef --version CHEF_VERSION --locked \
    && cargo install cargo-leptos --version 0.3.7 --locked
RUN rustup target add wasm32-unknown-unknown
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Cook the server dependencies with the same features cargo-leptos builds the bin with.
RUN cargo chef cook --release --no-default-features --features ssr --recipe-path recipe.json
COPY . .
RUN cargo leptos build --release
# Pre-compress static assets (the server serves the .br/.gz siblings).
RUN for f in target/site/pkg/*.wasm target/site/pkg/*.js target/site/pkg/*.css; do \
        brotli --best --keep --force "$f"; gzip --best --keep --force "$f"; \
    done

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /app/target/release/samete /app/samete
COPY --from=builder /app/target/site /app/site
COPY --from=builder /app/Cargo.toml /app/
WORKDIR /app
ENV LEPTOS_SITE_ROOT=site
ENV LEPTOS_SITE_ADDR=0.0.0.0:3000
ENV LEPTOS_ENV=PROD
EXPOSE 3000
CMD ["/app/samete"]
```
`CHEF_VERSION` = the exact version printed by `cargo search cargo-chef --limit 1` when you write the file (paste it in the report). No placeholder may remain (`grep -c CHEF_VERSION Dockerfile` → `0`).
2. `.github/workflows/ci.yml`: in the e2e job, `cargo binstall cargo-leptos --no-confirm` → `cargo binstall cargo-leptos@0.3.7 --no-confirm`. Add job:
```yaml
  docker:
    name: Docker image (build + boot smoke)
    runs-on: ubuntu-latest
    needs: check
    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_DB: samete
          POSTGRES_USER: samete
          POSTGRES_PASSWORD: samete
        ports:
          - 5432:5432
        options: >-
          --health-cmd "pg_isready -U samete -d samete"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    steps:
      - uses: actions/checkout@v4
      - name: docker build
        run: docker build -t samete:ci .
      - name: boot smoke
        run: |
          docker run -d --name samete --network host \
            -e DATABASE_URL=postgres://samete:samete@localhost/samete \
            -e SAMETE_SMS_DRY_RUN=true \
            -e LEPTOS_SITE_ADDR=127.0.0.1:3000 \
            samete:ci
          for i in $(seq 1 60); do curl -sf -o /dev/null http://127.0.0.1:3000/login && break; sleep 1; done
          curl -sf -o /dev/null http://127.0.0.1:3000/login || { docker logs samete; exit 1; }
          curl -sI -H 'Accept-Encoding: br' http://127.0.0.1:3000/pkg/samete.wasm | grep -qi 'content-encoding: br' \
            || { echo "wasm not served brotli"; exit 1; }
```
## Verification Gates
```bash
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))" && echo yaml-ok
grep -c "cargo-leptos@0.3.7\|cargo-leptos --version 0.3.7" .github/workflows/ci.yml Dockerfile
docker info > /dev/null 2>&1 && echo docker-available || echo docker-absent
```
**REQUIRED:** `yaml-ok`; counts `1` and `1`. If `docker-available`: run the two job steps locally with the branch + T0's two lines applied locally (uncommitted) → `/login` 200 and `content-encoding: br`. If `docker-absent` (this cloud container): report DONE_WITH_CONCERNS "docker gate pending CI". The orchestrator proves it with the CI `docker` job on the integrated branch; that run's green `docker` job is this unit's acceptance.

Commit: `ci(docker): build and boot the production image in CI; pin cargo-leptos`

---

# F-CI — Full unit tests, scheduled audit, UI invariants

## What You Must Do
1. `.github/workflows/ci.yml`:
   - `on:` add `workflow_dispatch:` and `schedule: [{cron: "17 6 * * 1"}]` (Mondays 06:17 UTC; advisories surface without a push).
   - Check job, after `cargo test`: steps
```yaml
      - name: cargo test (ssr — includes ssr-gated modules)
        run: cargo test --features ssr

      - name: UI invariants
        run: bash scripts/check-ui-invariants.sh

      - name: Future-incompat report (informational)
        run: cargo report future-incompatibilities || echo "no future-incompat report"
```
2. `scripts/check-ui-invariants.sh` (new, `chmod +x`):
```bash
#!/usr/bin/env bash
# Static UI invariants the compiler cannot see (conventions.md 2026-07-04 / frontend-protocol):
#  1. no `attr:`-prefixed ARIA attribute (leaks as a literal attribute on native elements)
#  2. every type="submit" carries the hydration gate within the next 8 lines
#  3. no data-testid selectors in production CSS
set -euo pipefail
fail=0

if grep -rnE 'attr:aria-' src/; then
    echo "FAIL: attr:-prefixed aria attribute (use the bare attribute)"; fail=1
fi

while IFS=: read -r file line _; do
    if ! sed -n "${line},$((line + 8))p" "$file" | grep -q '!hydrated.get()'; then
        echo "FAIL: $file:$line submit button without hydration gate"; fail=1
    fi
done < <(grep -rn 'type="submit"' src/)

if grep -rn 'data-testid' style/; then
    echo "FAIL: data-testid selector in style/"; fail=1
fi

[ "$fail" -eq 0 ] && echo "ui-invariants ok"
exit "$fail"
```
3. `justfile`: `test:` body → two lines `cargo test` and `cargo test --features ssr`; add recipe
```just
# Supply-chain audit (same command as CI)
audit:
    cargo audit
```
and make `check` run `fmt-check`, `clippy`, `test`, `audit` (keep its existing order, append `audit`). Install hint line in the recipe comment: `cargo binstall cargo-audit`.

## Verification Gates
```bash
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))" && echo yaml-ok
bash scripts/check-ui-invariants.sh > /tmp/fci-inv.log 2>&1; echo "inv=$?"; grep -c "attr:aria-" /tmp/fci-inv.log; grep -c "without hydration gate" /tmp/fci-inv.log
just --list | grep -c "audit"
```
**REQUIRED:** `yaml-ok`. On a base without A1: `inv=1`, attr count `13`, hydration-gate count `0`. The 13 lines are exactly A1's list, so the script proves A1's target and no other violation. With A1 merged locally (`git merge --no-edit <A1 branch>` if it exists): `ui-invariants ok`, `inv=0`. Sabotage: temporarily delete `|| !hydrated.get()` from one `type="submit"` site → the script names that file:line, `inv=1`; revert. `just --list` audit count ≥ `1`.

Commit: `ci: run ssr tests, weekly audit, and static UI invariants`

---

# F-GEN — Generation is transactional and excludes deactivated users

## What You Must Do (base includes U4's branch)
In `generate_assignments_action`:
1. Confirmed query → join active users:
```sql
        SELECT e.user_id
        FROM enrollments e
        JOIN users u ON u.id = e.user_id
        WHERE e.season_id = $1
          AND e.confirmed_ready_at IS NOT NULL
          AND u.status = 'active'
```
2. Order: phase check → confirmed query → `len() < 3` guard → algorithm + `validate_cycles` → `let mut tx = pool.begin().await.map_err(db_err)?;` → `DELETE FROM assignments WHERE season_id = $1` on `&mut *tx` → store rows on `&mut *tx` → `tx.commit().await.map_err(db_err)?`. `store_and_build_preview` takes `&mut sqlx::PgConnection` (call with `&mut tx`) instead of `&PgPool`; every statement inside uses it. A failed guard or validation now leaves existing assignments untouched.
3. sqlx prepare.
## Verification Gates
```bash
awk '/pub async fn generate_assignments_action/,/^}/' src/admin/assignments.rs | grep -n "DELETE FROM assignments\|< 3\|pool.begin\|commit"
```
**REQUIRED:** line order: `< 3` < `pool.begin` < `DELETE FROM assignments` < `commit`.
Standard gates (deltas 0). E2E ×3 delta 0 (titles `3.1 — admin generates assignments`, `3.3 — admin swaps two assignments`).
Commit: `fix(assignments): generate in one transaction from active confirmed participants`

---

# F-SMSF — SMS targets exclude deactivated users; assignment SMS only after release

## What You Must Do (`src/admin/sms.rs`; each count helper and its send query share the predicate)
1. `count_unnotified_senders` + `send_assignment_sms` targets: `JOIN users u ON u.id = a.sender_id` … `AND u.status = 'active'`.
2. `count_unconfirmed_enrolled` + `send_confirm_nudge_sms` targets: join users, `AND u.status = 'active'`.
3. `count_no_response_recipients` + `send_receipt_nudge_sms` targets: `JOIN users u ON u.id = a.recipient_id` … `AND u.status = 'active'`.
4. `send_assignment_sms` season query → `phase = 'delivery' AND launched_at IS NOT NULL`, error key `sms_error_no_delivery_season`; delete the comment about the 'assignment','delivery' predicate and the now-unused key `sms_error_no_assignment_delivery_season` (`grep -rn sms_error_no_assignment_delivery_season src` = 0 first; F-SMSF owns that key's deletion in `locales/uk.json` — add `locales/uk.json` to this lane's write-set).
5. sqlx prepare.
## Verification Gates
```bash
grep -c "u.status = 'active'" src/admin/sms.rs
grep -c "'assignment', 'delivery'" src/admin/sms.rs
```
**REQUIRED:** `8`, `0`. Standard gates (deltas 0). E2E ×3 delta 0 (titles `5.1 — admin triggers assignment SMS`, `5.2 — admin triggers receipt-nudge SMS`, `5.4 — admin triggers pre-deadline nudge SMS`).
Commit: `fix(sms): never target deactivated users; assignment SMS only in delivery`

---

# F-DEACT — Deactivation only applies to participants

## What You Must Do (`src/admin/participants.rs`)
Key (F-DEACT owns): `"participants_error_not_a_participant": "Деактивувати можна лише учасника."`.
`deactivate_participant`: the deactivating UPDATE gains `AND role = 'participant'`; if `rows_affected() == 0` → `Err(ServerFnError::new(td_string!(Locale::uk, participants_error_not_a_participant)))` and the session DELETE is not executed (move it after the check, same transaction as today or a new `pool.begin()` tx covering both statements). Admins (including the caller) are never `role = 'participant'`, so self- and admin-deactivation are impossible by construction.
## Verification Gates
Static proof (the predicate makes admin rows unreachable; the admin is never listed in the participant table, so no UI path exists to test):
```bash
awk '/pub async fn deactivate_participant/,/^}/' src/admin/participants.rs | grep -c "role = 'participant'"
awk '/pub async fn deactivate_participant/,/^}/' src/admin/participants.rs | grep -n "rows_affected\|DELETE FROM sessions"
```
**REQUIRED:** `1`; the `rows_affected` line number is lower than the `DELETE FROM sessions` line number. Standard gates (deltas 0). E2E ×3 delta 0 (title `6.1 — admin deactivates a participant`).
Commit: `fix(admin): deactivation targets participants only`

---

# F-ADV — Deadline-gated advance; cancel only after launch

## What You Must Do (base includes U4's branch)
Keys (F-ADV owns):
```json
"season_error_signup_deadline_not_passed": "Реєстрація ще триває — перейти далі можна після її завершення.",
"season_error_confirm_deadline_not_passed": "Підтвердження ще триває — перейти далі можна після дедлайну.",
"season_advance_deadline_hint": "Кнопка стане активною після дедлайну поточного етапу.",
"season_error_not_launched": "Скасувати можна лише запущений сезон."
```
### ADV.1 TDD — `src/admin/season.rs`
```rust
#[cfg(test)]
mod advance_tests {
    use super::{DeadlineGate, advance_deadline_gate};
    use crate::types::Phase;
    use time::{Duration, OffsetDateTime};

    fn t(h: i64) -> OffsetDateTime { OffsetDateTime::UNIX_EPOCH + Duration::hours(1000 + h) }

    #[test]
    fn enrollment_waits_for_signup_deadline() {
        assert_eq!(advance_deadline_gate(Phase::Enrollment, t(1), t(2), t(0), false), Some(DeadlineGate::Signup));
        assert_eq!(advance_deadline_gate(Phase::Enrollment, t(1), t(2), t(1), false), None);
    }

    #[test]
    fn preparation_waits_for_confirm_deadline() {
        assert_eq!(advance_deadline_gate(Phase::Preparation, t(-5), t(2), t(0), false), Some(DeadlineGate::Confirm));
        assert_eq!(advance_deadline_gate(Phase::Preparation, t(-5), t(2), t(3), false), None);
    }

    #[test]
    fn later_phases_have_no_deadline_gate() {
        for phase in [Phase::Assignment, Phase::Delivery, Phase::Complete, Phase::Cancelled] {
            assert_eq!(advance_deadline_gate(phase, t(9), t(9), t(0), false), None);
        }
    }

    #[test]
    fn test_mode_bypasses_gates() {
        assert_eq!(advance_deadline_gate(Phase::Enrollment, t(1), t(2), t(0), true), None);
    }
}
```
```rust
/// Which deadline still blocks advancing the season.
#[cfg(any(feature = "ssr", test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeadlineGate {
    Signup,
    Confirm,
}

/// ARCH: advance waits until the current phase's deadline has passed
/// (inclusive: `now >= deadline` passes). Test mode bypasses deadlines.
#[cfg(any(feature = "ssr", test))]
pub(crate) fn advance_deadline_gate(
    phase: crate::types::Phase,
    signup_deadline: time::OffsetDateTime,
    confirm_deadline: time::OffsetDateTime,
    now: time::OffsetDateTime,
    test_mode: bool,
) -> Option<DeadlineGate> {
    use crate::types::Phase;
    if test_mode {
        return None;
    }
    match phase {
        Phase::Enrollment if now < signup_deadline => Some(DeadlineGate::Signup),
        Phase::Preparation if now < confirm_deadline => Some(DeadlineGate::Confirm),
        Phase::Enrollment | Phase::Preparation | Phase::Assignment | Phase::Delivery | Phase::Complete | Phase::Cancelled => None,
    }
}
```
### ADV.2 Server
- `advance_season`: load `signup_deadline, confirm_deadline` with the season (extend the query or a second `query!`); before `UPDATE`: `if let Some(gate) = advance_deadline_gate(season.phase, s, c, OffsetDateTime::now_utc(), test_mode)` → `Err` with the matching key. `test_mode` computed the way the base computes it (env read; U2's `config.test_mode()` after merge).
- `cancel_season`: query adds `AND launched_at IS NOT NULL`; on `None` → `season_error_not_launched` if a non-terminal unlaunched season exists, else the existing `season_error_no_active_season` (two queries or one `query!` selecting `launched_at IS NOT NULL AS "launched!"`; pick the latter).
- `src/admin/state.rs` `AdminSeason` add `pub advance_waits_for_deadline: bool` computed in `get_admin_state` with the same fn (`is_some()`).
### ADV.3 UI (`src/admin/page.rs` `render_active_season`)
- Advance button `disabled` also when `season.advance_waits_for_deadline`; when true render `<p class="text-sm text-(--color-text-muted)" data-testid="advance-deadline-hint">{t!(i18n, season_advance_deadline_hint)}</p>` next to it.
- Cancel button (`cancel-button`) rendered only when `launched && !is_terminal` (today: `!is_terminal`).
### ADV.4 E2E
POM: `async expectCancelUnavailable() { await expect(this.page.getByTestId("cancel-button")).not.toBeVisible(); }`. Spec: insert after `"4.1 — admin creates a season"` (season exists, not launched):
```ts
    // Story 4.3 AC: cancel is only offered after launch.
    test("4.3 — cancel is not offered before launch", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.goToDashboard();
      await expect(page.getByTestId("launch-button")).toBeVisible();
      await app.expectCancelUnavailable();
    });
```
If the title `"4.1 — admin creates a season"` differs on base, anchor after the test that calls `createSeason` in the Epic 4 block and paste the exact title in the report.
## Verification Gates
Standard gates: bare +4, ssr +4. E2E ×3: delta +1; titles: new + `cancel — admin cancels the season` + every `phase — advance …` title (test mode keeps them green).
Commit: `fix(season): gate advance on deadlines and offer cancel only after launch`

---

# F-EDIT — Organizer edits season dates and theme

## What You Must Do
Keys (F-EDIT owns):
```json
"season_edit_title": "Змінити сезон",
"season_edit_save_button": "Зберегти зміни",
"season_edit_saving_loading": "Зберігаю...",
"admin_season_updated_toast": "Сезон оновлено!",
"season_error_edit_terminal": "Завершений або скасований сезон змінити не можна.",
"season_error_signup_locked": "Реєстрацію вже закрито — її дедлайн змінити не можна.",
"season_error_confirm_locked": "Етап підтвердження завершено — дедлайн змінити не можна."
```
(Reuses existing `season_error_signup_deadline_past`, `season_error_confirm_deadline_past`, `season_error_signup_after_confirm`.)
### EDIT.1 TDD — `src/admin/season.rs`
```rust
#[cfg(test)]
mod edit_tests {
    use super::{SeasonEditError, validate_season_edit};
    use crate::types::Phase;
    use time::{Duration, OffsetDateTime};

    fn t(h: i64) -> OffsetDateTime { OffsetDateTime::UNIX_EPOCH + Duration::hours(1000 + h) }
    const NOW: i64 = 0;

    #[test]
    fn enrollment_may_move_both_future_deadlines() {
        assert_eq!(validate_season_edit(Phase::Enrollment, (t(5), t(10)), (t(6), t(12)), t(NOW)), Ok(()));
    }

    #[test]
    fn preparation_may_extend_confirm_only() {
        assert_eq!(validate_season_edit(Phase::Preparation, (t(-5), t(10)), (t(-5), t(20)), t(NOW)), Ok(()));
        assert_eq!(validate_season_edit(Phase::Preparation, (t(-5), t(10)), (t(1), t(20)), t(NOW)), Err(SeasonEditError::SignupLocked));
    }

    #[test]
    fn assignment_locks_both_deadlines() {
        assert_eq!(validate_season_edit(Phase::Assignment, (t(-9), t(-1)), (t(-9), t(5)), t(NOW)), Err(SeasonEditError::ConfirmLocked));
        assert_eq!(validate_season_edit(Phase::Assignment, (t(-9), t(-1)), (t(-9), t(-1)), t(NOW)), Ok(()));
    }

    #[test]
    fn terminal_season_is_locked() {
        assert_eq!(validate_season_edit(Phase::Complete, (t(1), t(2)), (t(1), t(2)), t(NOW)), Err(SeasonEditError::Terminal));
        assert_eq!(validate_season_edit(Phase::Cancelled, (t(1), t(2)), (t(1), t(2)), t(NOW)), Err(SeasonEditError::Terminal));
    }

    #[test]
    fn changed_deadlines_must_be_future() {
        assert_eq!(validate_season_edit(Phase::Enrollment, (t(5), t(10)), (t(-1), t(10)), t(NOW)), Err(SeasonEditError::SignupPast));
        assert_eq!(validate_season_edit(Phase::Enrollment, (t(5), t(10)), (t(5), t(-1)), t(NOW)), Err(SeasonEditError::ConfirmPast));
    }

    #[test]
    fn signup_must_precede_confirm() {
        assert_eq!(validate_season_edit(Phase::Enrollment, (t(5), t(10)), (t(11), t(10)), t(NOW)), Err(SeasonEditError::SignupAfterConfirm));
    }
}
```
```rust
/// Why a season edit is refused.
#[cfg(any(feature = "ssr", test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SeasonEditError {
    Terminal,
    SignupLocked,
    ConfirmLocked,
    SignupPast,
    ConfirmPast,
    SignupAfterConfirm,
}

/// Validate a proposed (signup, confirm) pair against the current one.
/// A deadline may change only while its phase is open and it has not passed;
/// a changed deadline must be in the future; signup must precede confirm.
#[cfg(any(feature = "ssr", test))]
pub(crate) fn validate_season_edit(
    phase: crate::types::Phase,
    current: (time::OffsetDateTime, time::OffsetDateTime),
    proposed: (time::OffsetDateTime, time::OffsetDateTime),
    now: time::OffsetDateTime,
) -> Result<(), SeasonEditError> {
    use crate::types::Phase;
    if phase.is_terminal() {
        return Err(SeasonEditError::Terminal);
    }
    let signup_changed = proposed.0 != current.0;
    let confirm_changed = proposed.1 != current.1;
    if signup_changed {
        if phase != Phase::Enrollment || current.0 <= now {
            return Err(SeasonEditError::SignupLocked);
        }
        if proposed.0 <= now {
            return Err(SeasonEditError::SignupPast);
        }
    }
    if confirm_changed {
        if !matches!(phase, Phase::Enrollment | Phase::Preparation) || current.1 <= now {
            return Err(SeasonEditError::ConfirmLocked);
        }
        if proposed.1 <= now {
            return Err(SeasonEditError::ConfirmPast);
        }
    }
    if proposed.0 >= proposed.1 {
        return Err(SeasonEditError::SignupAfterConfirm);
    }
    Ok(())
}
```
### EDIT.2 Server fn
```rust
/// Update the active season's deadlines and theme (admin only).
///
/// # Errors
///
/// Returns `Err` if caller is not admin, there is no non-terminal season,
/// a deadline is malformed, or [`validate_season_edit`] refuses the change.
#[server]
pub async fn update_season(
    signup_deadline: String,
    confirm_deadline: String,
    theme: Option<String>,
) -> Result<(), ServerFnError>
```
Body: `require_admin`; parse both deadlines with the SAME closure as `create_season` (extract it to a private `#[cfg(feature = "ssr")] fn parse_deadline_input(s: &str) -> Result<OffsetDateTime, ServerFnError>` used by both); load `id, phase, signup_deadline, confirm_deadline` of the season `WHERE phase NOT IN ('complete','cancelled')`; none → `season_error_no_active_season`; map each `SeasonEditError` to its key (Terminal→`season_error_edit_terminal`, SignupLocked→`season_error_signup_locked`, ConfirmLocked→`season_error_confirm_locked`, SignupPast→`season_error_signup_deadline_past`, ConfirmPast→`season_error_confirm_deadline_past`, SignupAfterConfirm→`season_error_signup_after_confirm`); theme trimmed, empty → NULL (same as create); `UPDATE seasons SET signup_deadline = $1, confirm_deadline = $2, theme = $3 WHERE id = $4`.
### EDIT.3 State + UI
- `AdminSeason` add `pub signup_deadline_input: String, pub confirm_deadline_input: String` = UTC `YYYY-MM-DDTHH:MM` (format via `time::macros::format_description!("[year]-[month]-[day]T[hour]:[minute]")`), filled in `get_admin_state`.
- Admin season summary: add `data-testid="season-confirm-deadline"` to the confirm-deadline `<dd>`.
- `AdminPage`: `let update_action = ServerAction::<UpdateSeason>::new();` → add to `admin_state` source tuple, `action_error` chain, toast Effect (`admin_season_updated_toast`); thread into `render_active_season` (new param before `hydrated`).
- In `render_active_season`, when `!is_terminal`, render:
```rust
            <details data-testid="edit-season">
                <summary class="btn" data-variant="secondary" data-size="sm">{t!(i18n, season_edit_title)}</summary>
                <leptos::form::ActionForm action=update_action>
                    <div class="field">
                        <label class="field-label" for="edit-signup">{t!(i18n, season_signup_deadline_label)}</label>
                        <input class="field-input" id="edit-signup" type="datetime-local" name="signup_deadline"
                            value=signup_input data-testid="edit-signup-deadline-input" />
                    </div>
                    <div class="field">
                        <label class="field-label" for="edit-confirm">{t!(i18n, season_confirm_deadline_label)}</label>
                        <input class="field-input" id="edit-confirm" type="datetime-local" name="confirm_deadline"
                            value=confirm_input data-testid="edit-confirm-deadline-input" />
                    </div>
                    <div class="field">
                        <label class="field-label" for="edit-theme">{t!(i18n, season_theme_label)}</label>
                        <input class="field-input" id="edit-theme" type="text" name="theme" maxlength="100"
                            value=theme_input data-testid="edit-theme-input" />
                    </div>
                    <button class="btn" data-variant="secondary" type="submit" data-testid="save-season-button"
                        disabled=move || update_pending.get() || !hydrated.get()
                        aria-busy=move || update_pending.get().then_some("true")>
                        {move || if update_pending.get() { t!(i18n, season_edit_saving_loading).into_any() } else { t!(i18n, season_edit_save_button).into_any() }}
                    </button>
                </leptos::form::ActionForm>
            </details>
```
`value=` prefill is deliberate here: an edit form shows the current values. The conventions ban covers `value=""` on inputs Playwright fills before hydration; the POM below waits for hydration first. Cite this in a one-line comment.
### EDIT.4 E2E
POM:
```ts
  async editConfirmDeadline(isoMinute: string) {
    await this.page.goto("/admin");
    await this.page.getByTestId("edit-season").locator("summary").click();
    await expect(this.page.getByTestId("save-season-button")).toBeEnabled();
    await this.page.getByTestId("edit-confirm-deadline-input").fill(isoMinute);
    await this.clickAndWaitForResponse(this.page.getByTestId("save-season-button"), "update_season");
  }

  async expectSeasonConfirmDeadline(text: string) {
    await this.page.goto("/admin");
    await expect(this.page.getByTestId("season-confirm-deadline")).toContainText(text);
  }
```
(`locator("summary")` inside a testid scope: the `<summary>` is the only native disclosure control; comment that in the POM.) Spec, Cancel Season block, insert after `"setup — create and launch a new season"`:
```ts
    // O6: organizer extends the confirm deadline of a running season.
    test("4.1 — admin extends the confirm deadline", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      const extended = futureDeadline(30);
      await app.editConfirmDeadline(extended);
      await app.expectSeasonConfirmDeadline(formatDateUk(extended));
    });
```
`formatDateUk`: import from `capture-constants`; if absent on base, add it verbatim from PLAN-product §V.1 (this lane's write-set includes `capture-constants.ts`).
## Verification Gates
Standard gates: bare +6, ssr +6. E2E ×3: delta +1; title new.
Commit: `feat(season): let the organizer edit deadlines and theme`

---

# F-ALERT — Organizer SMS when a participant reports not received

## What You Must Do
Key (F-ALERT owns): `"sms_not_received_alert_body": "Саме Те: {{ name }} повідомив(ла), що не отримав(ла) лист. Деталі — в адмінці."`
1. `src/admin/sms.rs`:
```rust
/// Story 2.4: tell every active organizer that `participant_name` reported
/// not receiving mail. Best-effort: failures are logged, never surfaced to the
/// participant (their report is already stored).
#[cfg(feature = "ssr")]
pub(crate) async fn alert_organizers_not_received(pool: &sqlx::PgPool, participant_name: &str) {
    use crate::{config::Config, i18n::i18n::{Locale, td_string}, sms};
    let (Some(config), Some(client)) = (
        leptos::context::use_context::<Config>(),
        leptos::context::use_context::<reqwest::Client>(),
    ) else {
        tracing::error!("not-received alert: config/http client missing from context");
        return;
    };
    let phones = match sqlx::query_scalar!(
        r#"SELECT phone FROM users WHERE role = 'admin' AND status = 'active'"#
    )
    .fetch_all(pool)
    .await
    {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(error = %e, "not-received alert: admin lookup failed");
            return;
        }
    };
    let body = td_string!(Locale::uk, sms_not_received_alert_body, name = participant_name).to_string();
    for phone in phones {
        if let Err(e) = sms::send_sms(&config, &client, &phone, &body).await {
            tracing::warn!(phone = %phone, error = %e, "not-received alert SMS failed");
        }
    }
}
```
(If `td_string!` interpolation syntax differs, use the form `td_string!` documents for `{{ name }}` keys; check the Leptos MCP `get-documentation` or leptos_i18n docs. No `format!` on the key.)
2. `src/pages/home.rs` `confirm_receipt`: after the UPDATE succeeds, add
```rust
    if new_status == ReceiptStatus::NotReceived {
        crate::admin::sms::alert_organizers_not_received(&pool, &user.name).await;
    }
```
(`user` = the `require_auth` user; if its struct lacks `name`, select `name` with one `query_scalar!`.)
## Verification Gates
The isolated harness runs the server in the foreground of its own output, so dry-run SMS lines land in the E2E log: after an E2E run, `grep -c "повідомив(ла), що не отримав(ла) лист" /tmp/e2e_falert_1.log` → **REQUIRED:** ≥ `1` (dry-run logs every SMS; `2.4 — participant reports not received with note` triggers it). Standard gates (deltas 0). E2E ×3 delta 0.
Commit: `feat(receipt): SMS the organizer when a participant reports mail not received`

---

# F-HOME — Enrollment closed, theme persists, how-it-works, content rules, countdown

## What You Must Do
Keys (F-HOME owns):
```json
"home_enrollment_closed_heading": "Реєстрацію закрито",
"home_enrollment_closed_body": "Цього сезону реєстрація вже завершилась. Коли відкриється наступний — надішлемо SMS.",
"home_how_it_works_heading": "Як це працює",
"home_how_it_works_step_1": "Реєструєшся на сезон і створюєш щось своє — лист, малюнок, колаж.",
"home_how_it_works_step_2": "Підтверджуєш, що лист готовий, до дедлайну.",
"home_how_it_works_step_3": "Отримуєш адресу незнайомця з клубу й надсилаєш лист Новою Поштою. Хтось інший надсилає лист тобі.",
"home_how_it_works_step_4": "Зустрічаємося всі разом і розповідаємо, що отримали.",
"home_guideline_rules_1": "Без незаконного, погроз і навмисно образливого.",
"home_guideline_rules_2": "Отриманий лист лишається в отримувача назавжди.",
"home_guideline_rules_3": "Не публікуй і не фотографуй чужі листи без згоди автора.",
"countdown_days_one": "Залишився {{ n }} день",
"countdown_days_few": "Залишилося {{ n }} дні",
"countdown_days_many": "Залишилося {{ n }} днів"
```
### HOME.1 TDD — `src/date_format.rs` (identical block also used by F-ADMCD)
```rust
/// Ukrainian plural class for a day count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DayCount {
    One,
    Few,
    Many,
}

/// Whole days left until `deadline` (rounded up); `None` once passed.
pub fn days_remaining(now: time::OffsetDateTime, deadline: time::OffsetDateTime) -> Option<i64> {
    if deadline <= now {
        return None;
    }
    let secs = (deadline - now).whole_seconds();
    Some((secs + 86_399) / 86_400)
}

/// Ukrainian plural rule: 1, 21, 31… → One; 2–4, 22–24… → Few; else Many (incl. 11–14).
pub fn day_count_class(n: i64) -> DayCount {
    let (m10, m100) = (n % 10, n % 100);
    if m10 == 1 && m100 != 11 {
        DayCount::One
    } else if (2..=4).contains(&m10) && !(12..=14).contains(&m100) {
        DayCount::Few
    } else {
        DayCount::Many
    }
}

#[cfg(test)]
mod tests {
    use super::{DayCount, day_count_class, days_remaining};
    use time::{Duration, OffsetDateTime};

    #[test]
    fn days_round_up_and_stop_at_deadline() {
        let now = OffsetDateTime::UNIX_EPOCH;
        assert_eq!(days_remaining(now, now + Duration::hours(1)), Some(1));
        assert_eq!(days_remaining(now, now + Duration::hours(25)), Some(2));
        assert_eq!(days_remaining(now, now), None);
        assert_eq!(days_remaining(now, now - Duration::hours(1)), None);
    }

    #[test]
    fn ukrainian_plural_classes() {
        for n in [1, 21, 31, 101] { assert_eq!(day_count_class(n), DayCount::One, "{n}"); }
        for n in [2, 3, 4, 22, 24] { assert_eq!(day_count_class(n), DayCount::Few, "{n}"); }
        for n in [5, 9, 11, 12, 14, 20, 25, 111] { assert_eq!(day_count_class(n), DayCount::Many, "{n}"); }
    }
}
```
(`date_format` is ssr-gated → these run under `--features ssr`.)
### HOME.2 States (`src/pages/home.rs`)
- New variant `EnrollmentClosed` (doc: "Enrollment phase, not enrolled, signup deadline passed; organizer has not advanced yet."). Pure fn + tests:
```rust
/// Enrollment-phase state for a participant who is not enrolled.
#[cfg(any(feature = "ssr", test))]
fn unenrolled_enrollment_state(deadline_passed: bool, open: HomeState) -> HomeState {
    if deadline_passed { HomeState::EnrollmentClosed } else { open }
}
```
tests: `true` → `EnrollmentClosed`; `false` → the passed `open` value (`matches!(…, HomeState::EnrollmentOpen { .. })`). In `resolve_enrollment_state`'s not-enrolled branch: `deadline_passed = is_past_deadline(season.signup_deadline, test_mode)` (test_mode as the base computes it) → `Ok(unenrolled_enrollment_state(deadline_passed, HomeState::EnrollmentOpen { … }))`.
- `Enrolled` and `Preparing` gain `theme: Option<String>` (from `season.theme`); render `{theme.map(|v| view!{ <p class="overline-label" data-testid="season-theme">{t!(i18n, home_theme_label)}{v}</p> })}` under the heading in both.
- `Enrolled` and `Preparing` gain `days_left: Option<i64>` = `date_format::days_remaining(now, season.confirm_deadline)` (Preparing: only when `!deadline_passed`); render next to the deadline:
```rust
{days_left.map(|n| view! { <p class="deadline" data-testid="deadline-countdown">{countdown_label(n, i18n)}</p> })}
```
with
```rust
fn countdown_label(n: i64, i18n: leptos_i18n::I18nContext<crate::i18n::i18n::Locale>) -> AnyView {
    use crate::date_format::{DayCount, day_count_class};
    match day_count_class(n) {
        DayCount::One => t!(i18n, countdown_days_one, n = n).into_any(),
        DayCount::Few => t!(i18n, countdown_days_few, n = n).into_any(),
        DayCount::Many => t!(i18n, countdown_days_many, n = n).into_any(),
    }
}
```
NOTE: `date_format` is ssr-gated but `DayCount`/`day_count_class` are needed by the client render. Move the `#[cfg(feature = "ssr")]` in `src/lib.rs` from `pub mod date_format;` onto `format_date_uk` alone (it uses nothing client-incompatible; the gate exists only because no client code used the module). `days_remaining` stays callable on both. F-ADMCD carries the identical edit; integration keeps one.
- `EnrollmentOpen` render: after the guideline paragraph:
```rust
        <ul class="text-sm" data-testid="season-guideline-rules">
            <li>{t!(i18n, home_guideline_rules_1)}</li>
            <li>{t!(i18n, home_guideline_rules_2)}</li>
            <li>{t!(i18n, home_guideline_rules_3)}</li>
        </ul>
```
- `EnrollmentClosed` render: `.empty-state` with `data-testid="enrollment-closed"`, headline/body keys above.
- `NoSeason` render: after the body:
```rust
                <section class="card" data-testid="how-it-works">
                    <h2 class="overline-label">{t!(i18n, home_how_it_works_heading)}</h2>
                    <ol>
                        <li>{t!(i18n, home_how_it_works_step_1)}</li>
                        <li>{t!(i18n, home_how_it_works_step_2)}</li>
                        <li>{t!(i18n, home_how_it_works_step_3)}</li>
                        <li>{t!(i18n, home_how_it_works_step_4)}</li>
                    </ol>
                </section>
```
- sqlx prepare.
### HOME.3 E2E
POM: `expectGuidelineRules()` (`season-guideline-rules` visible), `expectThemeShown(theme)` (`season-theme` contains), `expectCountdown()` (`deadline-countdown` visible). Spec:
- `"2.1 — content guidelines visible during enrollment"`: append `await app.expectGuidelineRules();`.
- Insert after `"2.1 — participant enrolls in season"`: test `"2.1 — theme and countdown stay visible after enrolling"`: login A, goHome, `expectThemeShown(SEASON_THEME)`, `expectCountdown()`.
- EnrollmentClosed / how-it-works: test mode bypasses deadlines, and the main chain never shows NoSeason to a participant. Cover them with unit tests (above) and pixels: in `visual-audit.spec.ts` test `"capture home — no season"` add `await expect(page.getByTestId("how-it-works")).toBeVisible();` before its capture (this lane's write-set includes `visual-audit.spec.ts`).
## Verification Gates
Standard gates: bare +2 (`unenrolled_enrollment_state` tests), ssr +4 (+2 date_format). E2E ×3: delta +1; titles new + `2.1 — content guidelines visible during enrollment` + `capture home — no season`. Pixels: `home-no-season`, `home-enrollment-available`, `home-enrolled`, `home-confirm-ready-available` (4 mode dirs).
Commit: `feat(home): enrollment-closed state, persistent theme, countdown, rules and how-it-works`

---

# F-ADMCD — Admin countdown and per-participant failure history

## What You Must Do
Keys (F-ADMCD owns): `"participants_table_failed": "Не надіслав (сезонів)"`, plus the three `countdown_days_*` keys with the SAME strings as F-HOME (identical key/value → union is a no-op).
1. `src/date_format.rs` + `src/lib.rs`: identical to F-HOME §HOME.1 and its cfg move.
2. `AdminSeason` add `pub signup_days_left: Option<i64>, pub confirm_days_left: Option<i64>` (computed in `get_admin_state` with `days_remaining(now, …)`); render in the season summary `<dd>`s, after each date: `{days.map(|n| view!{ <span class="text-sm text-(--color-text-muted)" data-testid="admin-signup-countdown"> " · " {countdown_label(n, i18n)}</span> })}` (and `admin-confirm-countdown`). `countdown_label`: the same fn as F-HOME, placed in `src/admin/page.rs` (private; integration may dedupe into a shared component — not required).
3. `ParticipantSummary` add `pub failed_sends: i64`. A failed send by X = an assignment row whose sender is X and whose recipient reported `not_received`. `list_participants` select list adds exactly:
```sql
            (SELECT COUNT(*) FROM assignments a
             WHERE a.sender_id = users.id AND a.receipt_status = 'not_received') AS "failed_sends!"
```
(`receipt_status` on an assignment row is what that row's recipient reported.) New column header `participants_table_failed`, cell `<td data-testid="participant-failed-sends">{p.failed_sends}</td>` before the actions column.
4. sqlx prepare.
## Verification Gates
E2E: insert after `"2.4 — admin sees not-received alert"`: test `"6.1 — participant list shows failed sends per participant"`. Admin dashboard: the row of `await app.cycleSenderNameFor(NAMES.B)` has `participant-failed-sends` text `1`. `cycleSenderNameFor` belongs to P-F: if absent on base, copy it verbatim from PLAN-product §F.4 together with U4's `cycleSenderId` (which needs U4's `cycle-link` testids). If U4's testids are absent on base, merge `worktree-agent-a46c4c68440f8d984` into this lane's base (that branch exists). Also assert `admin-confirm-countdown` is visible in the earlier `"dashboard — enrolled count visible"` test (append one line).
Standard gates: bare +0 (date_format tests ssr), ssr +2 (if F-HOME is not yet merged; the integrator dedupes). E2E ×3: delta +1.
Commit: `feat(admin): deadline countdown and failed-send history`

---

# F-COPY — Toasts name the next SMS action

## What You Must Do
`locales/uk.json` values only:
```json
"admin_season_launched_toast": "Сезон запущено! Тепер надішли SMS про відкриття — кнопка нижче.",
"admin_season_advanced_toast": "Етап змінено! Перевір SMS-дії нижче."
```
Gate: `grep -rn "Сезон запущено!\|Фазу просунуто!" end2end/tests` → 0 matches before editing (no test asserts old text). Standard gates; E2E ×1 delta 0.
Commit: `fix(i18n): toasts point the organizer to the next SMS action`

---

# F-NAV — Logout on the onboarding page (Story 1.4)

## What You Must Do
`src/app.rs` `HeaderNav`: `show_nav` → `move || pathname.get() != "/login"`. E2E: POM `async expectLogoutAvailable() { await expect(this.page.getByTestId("logout-button")).toBeVisible(); }`; spec `"1.3 — first login redirects to onboarding"`: append `await app.expectLogoutAvailable();`.
Gates: standard (deltas 0); E2E ×3 delta 0; pixels `onboarding-*` (4 dirs).
Commit: `fix(nav): offer logout during onboarding`

---

# F-GEOM — Geometric assertions on every capture

## What You Must Do
1. `end2end/tests/fixtures/geometry.ts` (new):
```ts
import { type Page, expect } from "@playwright/test";

/**
 * Layout invariants checked on every visual capture.
 * page.evaluate is the only way to read layout metrics; expect.poll makes the
 * read web-first (retries until stable), which is what the README ban protects.
 */
export async function expectNoHorizontalOverflow(page: Page) {
  await expect
    .poll(() => page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth))
    .toBeLessThanOrEqual(0);
}

/** Every visible testid element lies inside the viewport horizontally (scroll containers excepted). */
export async function expectTestidsInsideViewport(page: Page) {
  await expect
    .poll(() =>
      page.evaluate(() => {
        const width = document.documentElement.clientWidth;
        return [...document.querySelectorAll("[data-testid]")]
          .filter((el) => {
            const r = el.getBoundingClientRect();
            if (r.width === 0 || r.height === 0) return false;
            if (el.closest(".sr-only, .data-table-wrapper")) return false;
            return r.left < -1 || r.right > width + 1;
          })
          .map((el) => el.getAttribute("data-testid"));
      }),
    )
    .toEqual([]);
}
```
2. `visual-audit.spec.ts`: in `captureState`, before each `recordScreenshot` call, call both helpers (all four modes).
3. If an assertion fails on a real defect (the poll output names the testid/state): fix it with the narrowest rule in `style/components.css`. Bind to `guidance/ui-review-prompt.md` form-first: question the layout, not the width. Report each fix (state, testid, rule). If more than 3 distinct defects appear, STOP and report the list (BLOCKED — the remaining fixes need design review).
## Verification Gates
Sabotage: add `min-width: 2000px` to `.prose-page` temporarily → one capture fails with a non-empty overflow; revert. E2E ×3 (full) delta 0; `grep -c "expectNoHorizontalOverflow" end2end/tests/visual-audit.spec.ts` ≥ `1`.
Commit: `test(visual): assert no horizontal overflow or off-viewport elements on every capture`

---

# F-SEED — Explicit conflict targets in cohort seed

## What You Must Do
`end2end/tests/fixtures/cohort-seed.sql`: the 5 `ON CONFLICT DO NOTHING` → `users`: `ON CONFLICT (id) DO NOTHING`; `delivery_addresses`: `(user_id)`; `seasons`: `(id)`; `enrollments`: `(user_id, season_id)`; `assignments`: `(season_id, sender_id)`. Replace the header comment block that justifies the bare form with: "Conflict targets name the row identity; a different unique violation (e.g. a phone already used by another user) now fails loudly — that means the seed is running against the wrong DB."
## Verification Gates
```bash
export SDB=postgres://samete:samete@localhost:5432/samete_fseed
DATABASE_URL=$SDB sqlx database drop -y; DATABASE_URL=$SDB sqlx database create; DATABASE_URL=$SDB sqlx migrate run
psql -v ON_ERROR_STOP=1 $SDB -f end2end/tests/fixtures/cohort-seed.sql | grep -c "INSERT 0 [1-9]"
psql -v ON_ERROR_STOP=1 $SDB -f end2end/tests/fixtures/cohort-seed.sql | grep -c "INSERT 0 0"
DATABASE_URL=$SDB sqlx database drop -y
grep -c "ON CONFLICT DO NOTHING" end2end/tests/fixtures/cohort-seed.sql
```
**REQUIRED:** first run `5`, second run `5` (idempotent), bare count `0`. Cohort capture: `VISUAL_SPEC=tests/visual-audit-cohort.spec.ts bash scripts/isolated-capture.sh fseed visual` → exit 0.
Commit: `test(seed): name conflict targets in cohort seed`

---

# F-COV — Coverage for behaviour a-scope found untested

## What You Must Do (spec + POM only; every testid exists at base)
POM additions (each a two-liner on the named testid):
- `expectInviteDistributor(code, name)` — `invite-code-row` filtered by `invite-code-cell` hasText `code` → `invite-code-distributor-cell` toContainText `name`.
- `expectInviteRedeemer(code, name)` — same row → `invite-code-redeemer-cell` toContainText `name`; and `invite-code-revoke-button` not visible in that row.
- `filterInviteCodes(text)` + `expectInviteRowCount(n)` — fill `invite-code-filter-input`; `invite-code-row` toHaveCount(n).
- `expectNotOnAdmin()` — `page.goto("/admin")`; `expect(page).not.toHaveURL(/\/admin/)`.
Tests:
1. after `"1.5 — generated codes appear in admin list with unused status"`: `"1.5 — invite list records the distributor"` → `expectInviteDistributor(CODES.A, "Організатор")`.
2. after `"1.1 — used invite code is rejected"`: `"1.6 — used code shows its redeemer and cannot be revoked"` → admin login, goToDashboard, `expectInviteRedeemer(CODES.A, NAMES.A)`.
3. same block: `"1.6 — invite codes can be filtered"` → admin, goToDashboard, `filterInviteCodes(CODES.B)`, `expectInviteRowCount(1)`.
4. `"5.2 — admin triggers receipt-nudge SMS"` body: before trigger `expectSmsCount("sms-count-no-response", "3")`; after `expectSmsReport()` `expectSmsCount("sms-count-no-response", "0")`.
5. before the Epic 4 test that calls `createSeason`: `"4.1 — create-season rejects a past signup deadline"` → admin, goToDashboard, fill `signup-deadline-input` with `futureDeadline(-1)`, `confirm-deadline-input` with `futureDeadline(21)`, click `create-season-button` → `signup-deadline-error` visible (POM `attemptCreateSeason(signup, confirm)` without the success wait).
6. after `"1.3 — returning participant skips onboarding"`: `"4.5 — participant cannot open the admin page"` → login A, `expectNotOnAdmin()`.
7. Session Management block, after the participant logout test: `"1.4 — old session cookie is rejected after logout"`:
```ts
    const app = new MailClubPage(page);
    await app.login(PHONES.A);
    const before = await page.context().cookies();
    await app.logout();
    await page.context().addCookies(before.filter((c) => c.name === "session"));
    await app.expectNoSession(); // P-V POM method; copy verbatim from PLAN-product §V.2 if absent
```
8. POM comment at `goHome` that says "15s navigation timeout" → "30s `navigationTimeout`" (e#9).
## Verification Gates
Mutation: locally comment out `auth::delete_session(...)` in the `logout` server fn (never commit) → test 7 fails; revert, `git diff` empty. Standard gates (deltas 0). E2E ×3: delta +6.
Commit: `test(e2e): cover invite list, filter, nudge counts, validation, admin guard and session revocation`

---

# DOC-OPUS lane (one opus agent; one commit per unit; base default)

Binding: read the affected doc fully before editing; describe behaviour exactly as this plan and PLAN-product.md / PLAN-blockers.md specify it (contracts). Never hard-code a number that another unit changes (test counts).

### F-DOC-README (`README.md`)
- Dependencies: delete blake2 and every CSRF mention; list `sha2` + `subtle` (OTP/session hashing, constant-time compare), `base64`, `leptos-use`, `leptos_i18n`; uuid features `v4, serde` (`js` comes via the `hydrate` feature).
- Project tree: add `src/invite_codes.rs`, `src/admin/invite_codes.rs`, `src/components/{stepper,toast,skeleton}.rs`, `scripts/`.
- Recipes table: add `capture-isolated`, `fmt`, `fmt-check`, `audit`; `test` runs bare + ssr.
- Dev setup: `cargo binstall cargo-leptos@0.3.7` (matches CI/Dockerfile); `cd end2end && npm ci && npx playwright install chromium`; ≥20 GB free disk for `target/`.
- Deployment checklist (Coolify): Postgres 16 service provisioned in Coolify + `DATABASE_URL`; enable Coolify scheduled DB backups; health check path `/login`; HSTS at Traefik; env `SAMETE_ADMIN_PHONE/NAME` (U3), `SAMETE_SITE_URL` (P-S), `SAMETE_CLIENT_IP_SOURCE=x-forwarded-for` (F-AUTH); never set `SAMETE_TEST_MODE`/`SAMETE_SMS_DRY_RUN` (U2); image sets `LEPTOS_ENV=PROD` (F-DOCKER).
- Organizer operations: SQL recipes for known groups (`INSERT INTO known_groups (name, weight) …`, `INSERT INTO known_group_members …`, delete member), per Product Spec §App Scope (D-F6).
Gate: `grep -ciE "blake2|csrf" README.md` → `0`.
Commit: `docs(readme): correct dependencies, recipes, setup and deployment checklist`

### F-DOC-CLAUDE (`CLAUDE.md`)
- Key Commands `just test`: "unit tests (bare + `--features ssr`)", no counts. `just clippy`: "clippy SSR + wasm hydrate". Add `just audit`.
- E2E pitfalls: navigationTimeout 30s (`playwright.config.ts`).
- WASM note: replace "471KB" with the brotli size measured now (`cargo leptos build --release && brotli -c -q 11 target/site/pkg/samete.wasm | wc -c`, with T0's two lines applied locally if not on base). Write "≈N KB (measured 2026-10-0X)".
- Leptos MCP + QA bridge files: mark "local developer environment only; absent in cloud containers".
Commit: `docs(claude): fix stale counts, timeouts and environment notes`

### F-DOC-GUIDE (`guidance/dev-protocol.md`, `guidance/frontend-protocol.md`, `end2end/README.md`)
- dev-protocol: "What to Test Where" — unit: pure decision fns (OTP send limits, lockout, client IP, phase/advance/edit rules, assignment algorithm, phone normalization); E2E: DB-bound OTP/session lifecycle. Lint snippet adds `priority = -1`. New §"Gating pure helpers" — the WHY that F-CFG comments point to: "`#[cfg(any(feature = \"ssr\", test))]` = server-only helper that bare `cargo test` must also compile; never use it for values rendered on both SSR and client (hydration mismatch)".
- frontend-protocol: z-index map → only grain `z-index: 1` exists; future layers get documented when added. `!important` ban: add the reduced-motion exception (`components.css` reduced-motion block).
- e2e README: navigationTimeout 30_000; Banned `waitForTimeout` — sanctioned exception `paintSettle` (capture-constants); Banned `page.evaluate` — exception `fixtures/geometry.ts` via `expect.poll`; CI runs the suite through `assert-playwright-ran.sh` (T0).
Commit: `docs(guidance): align protocols with code and sanctioned exceptions`

### F-DOC-SPEC (`spec/product/Product Spec.md`, `spec/technical/User Stories.md`, `spec/technical/Architecture.md`)
- Product Spec: Entry → "organizer meets the person and hands over a one-time invite code; the person self-registers with it after SMS verification"; App Scope "Registration (invite-code self-registration)"; Deletion unchanged.
- Stories: 4.2 "Launch opens enrollment; the organizer then sends the season-open SMS (5.3) from the same page"; 5.4 "organizer sends ~1 h before the deadline"; 5.2 "organizer sends ~5 days after release"; 2.4 "receipt prompt available from the start of delivery; reversible not-received (late arrival)"; 2.2 countdown = days-remaining label; 4.3 cancel after launch (unchanged), edits per F-EDIT.
- Architecture: §Invite-code-gated registration — deactivated phones receive an OTP like any phone and are refused after verification (uniform `request_otp`); §Rate Limiting table = F-AUTH constants (+ per-IP 10/h, global 50/h, lockout 10 failed/h, `auth_events`); admin dashboard rows — "Release" = advance to Delivery, assignment SMS sent separately; preparation/enrollment advance disabled until the phase deadline (F-ADV); delivery row → roster + forwarding (P-R/P-F).
Commit: `docs(spec): align product, stories and architecture with implemented behaviour`

### F-MANI (`.manifestos.yaml`)
- Fix `simlpe-made-easy` → `simple-made-easy`.
- Every element gets `source: /tmp/claude-manifesto-repo/LLM_MANIFESTOS/manifestos/<name>.md` (manifesto elements) or keeps its skill source.
- `subagents:` add keys for every dispatched type: `general-bound`, `code-quality-reviewer`, `Explore`, and `other:` catch-all (`stop-yapping`, `first-principles`, `simple-made-easy`, `kiss`); keep `implementer`, `spec-reviewer`.
Gate:
```bash
H=$(ls -d /root/.claude/plugins/cache/my-claude-skills/manifesto/*/hooks | tail -1)
for t in general-bound code-quality-reviewer Explore some-unknown-type; do
  echo "{\"agent_type\":\"$t\"}" | CLAUDE_PROJECT_DIR=$PWD bash $H/subagent-start.sh | grep -c "stop-yapping"
done
```
**REQUIRED:** each ≥ `1`.
Commit: `chore(manifestos): bind every subagent type with explicit sources`

### F-CFG (comments in `src/**` + dev-protocol §Gating, written above)
Every `#[cfg(any(feature = "ssr", test))]` line gets, on the line above, `// WHY: server-only helper also compiled for bare unit tests — see guidance/dev-protocol.md §Gating pure helpers`. If the preceding line is already a doc comment block, insert the WHY line between the doc block and the attribute.
Gate: `grep -rc 'cfg(any(feature = "ssr", test))' src | awk -F: '{s+=$2} END{print s}'` equals `grep -rc 'Gating pure helpers' src | awk -F: '{s+=$2} END{print s}'`. Run this unit LAST in the DOC lane and re-run the gate at integration of every code unit (new sites from other lanes get the same comment from the integrator).
Commit: `docs(code): explain the ssr-or-test gate at every site`

### F-UP (`orchestration_log/history/2026-10-04/plans/upstream-leptos-i18n-issue.md`)
Issue draft for `Baptistemontan/leptos_i18n`: title "SSR isomorphic effect reads disposed locale signal in detached task (context.rs:213)"; version 0.6.1/0.6.2; repro summary (concurrent client-aborted SSR requests); backtrace excerpt from `recon/2026-10-04/readiness/logs/n-stress-abort-debuginfo-backtrace.log` (≤30 lines); proposed `try_get()` diff (n-panic.md §Fix proposal). Filing is a public action under the owner's GitHub identity → the orchestrator files it (`gh issue create -R Baptistemontan/leptos_i18n --title … --body-file …`) only with the owner's explicit go-ahead, then records the URL in the file.
Commit: `docs(upstream): draft leptos_i18n disposed-signal issue`

---

# ORCH lane (orchestrator; not delegated)

### F-OPS
On the machine whose Postgres holds it: `psql "postgres://samete:samete@localhost:5432/postgres" -c 'DROP DATABASE IF EXISTS samete_ssr_debug2;'` then `psql … -tAc "select count(*) from pg_database where datname='samete_ssr_debug2'"` → `0`. In this cloud container, run the check; if the DB is absent here, the item still applies to the owner's Docker Postgres — run it there at the next local session (record in session.md).

### F-MEM (memory work — orchestrator writes by hand, conventions 2026-07-13)
`orchestration_log/reference/deferred_items.md`: delete (resolve = delete) — `implementation_plan.md` decision (file absent); "create-form unreachable" (stale); and, as each fixing unit integrates: IP rate limiting (F-AUTH), geometric assertions (F-GEOM), cfg WHY (F-CFG), orphan DB (F-OPS), cohort-seed (F-SEED), manifesto hook (F-MANI). T1 deletes the panic item itself.
`orchestration_log/reference/codebase_state.md` at session close: correct "SHIPPED, CI GREEN" (false-green until T0), E2E counts (118 + new tests, measured), "all submit buttons aria-busy" (true after A1), module inventory (+ components, `auth_events`, `date_format` scope).

---

## Forbidden Patterns (all F lanes)
- BANNED: waiting on another lane, or a "must be integrated first" step. Write against the contracts in §3.
- BANNED: committing the local `recursion_limit` lines.
- BANNED: `let _ =` on writes in auth paths (F-AUTH); best-effort SMS in F-ALERT logs every failure explicitly.
- BANNED: trusting any `X-Forwarded-For` hop other than the last, or trusting XFF when `SAMETE_CLIENT_IP_SOURCE=peer`.
- BANNED: distinct responses per phone from `request_otp` (F-AUTH-ORACLE gate is byte-identity).
- BANNED: `#[cfg]` inside `view!`; bare `>` in unbraced view attributes; `attr:` on native elements; wildcard arms on `HomeState`/`Phase`/`RosterStatus` in new matches; `format!()` class names; testid selectors in CSS.
- BANNED E2E: `waitForTimeout`, `networkidle`, `waitForLoadState`, `force: true`, `getByText`, `getByRole` with name, CSS-class selectors; `page.evaluate` outside `fixtures/geometry.ts`; imports from `@playwright/test` in spec files (fixtures may).
- BANNED commands: `just e2e*`, `just db-reset`, `_kill-stale`, port 3000 / DB `samete`, bare `cargo sqlx prepare`, `git push`, merges into the working branch, `[patch.crates-io]`.
- BANNED lint escapes without a one-line WHY; every new `pub fn` returning `Result` has `# Errors`.

## Definition of Done (per unit)
1. Base recipe followed (§3); pre-edit baseline pasted.
2. Every step applied; unit gates show REQUIRED output (pasted).
3. Standard gates green with stated deltas; `.sqlx/` regenerated when queries changed.
4. E2E gate ×3 where listed (×1 for copy units), named titles passed; pixels listed for UI units.
5. `git diff --stat <base>...HEAD` = the lane write-set; tree clean; one-line commit per unit; SHA reported.
6. DONE / DONE_WITH_CONCERNS / BLOCKED with evidence.

## Global Gates (orchestrator, on the integrated branch containing every F-unit)
```bash
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0
SQLX_OFFLINE=true cargo clippy --no-default-features --features ssr -- -D warnings
SQLX_OFFLINE=true cargo clippy --target wasm32-unknown-unknown --features hydrate --no-default-features -- -D warnings
cargo test && SQLX_OFFLINE=true cargo test --features ssr
bash scripts/check-ui-invariants.sh
python3 - <<'EOF'
import json, re, pathlib
keys = json.load(open('locales/uk.json'))
src = "".join(p.read_text() for p in pathlib.Path('src').rglob('*.rs'))
print([k for k in keys if not re.search(r'\b' + re.escape(k) + r'\b', src)])
EOF
bash scripts/isolated-capture.sh e2e_f_final full > /tmp/e2e_f_final.log 2>&1; echo "exit=$?"
grep -c "" orchestration_log/reference/deferred_items.md
```
**REQUIRED:** clippy clean ×2; tests ok, 0 failed; `ui-invariants ok`; orphan list `[]`; `exit=0`, 0 failed, 2 skipped; E2E passed = pre-plan main + **11** (F-AUTH 1, F-ADV 1, F-EDIT 1, F-HOME 1, F-ADMCD 1, F-COV 6). CI on the integrated branch: Check, E2E and `docker` jobs green (run via `workflow_dispatch`). `deferred_items.md` `## Open` section is empty (every item deleted by F-MEM as its unit integrated).

## Plan self-review
- Every §1 row names a unit or a §2 decision. No "deferred"/"unplanned" row.
- Contracts used across lanes are fully specified in this file or the cited plan sections: `formatDateUk` (P §V.1), `expectNoSession` (P §V.2), `cycleSenderNameFor` (P §F.4), `cycleSenderId` (U4 branch), `days_remaining`/`DayCount` (F-HOME §HOME.1, duplicated verbatim in F-ADMCD), `SmsMode`/`test_mode()` (U2, call sites adapt at integration).
- Placeholders: `CHEF_VERSION` is resolved by the implementer at write time and gated to 0 occurrences.
