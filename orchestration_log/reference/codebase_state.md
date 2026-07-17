# Codebase State

Last updated: 2026-07-12

## Module Inventory

| Module | Path | Purpose | Tests | Status |
|--------|------|---------|-------|--------|
| App shell | `src/app.rs` | Routes, guards, current_user Resource | E2E | Stable |
| Auth | `src/auth.rs` | Session, OTP, require_auth | E2E | Stable |
| Login | `src/pages/login.rs` | OTP flow, verify, logout; self-registration with invite code for new phones | E2E | Stable |
| Onboarding | `src/pages/onboarding.rs` | Nova Poshta address | E2E | Stable |
| Home | `src/pages/home.rs` | Participant dashboard, season states (envelope removed, deadline_passed added) | E2E | Stable |
| Admin: Page | `src/admin/page.rs` | Unified single-page admin with phase-aware sections | E2E | Stable |
| Admin: State | `src/admin/state.rs` | AdminState/AdminSeason types, get_admin_state() | E2E | Stable |
| Admin: Season | `src/admin/season.rs` | Server fns only: create, launch, advance, cancel | E2E | Stable |
| Admin: Participants | `src/admin/participants.rs` | Deactivate (register form replaced by invite codes) | E2E | Stable |
| Admin: Invite Codes | `src/admin/invite_codes.rs` | Generate, list, revoke invite codes; client-side filter | E2E | Stable |
| Invite Codes | `src/invite_codes.rs` | Word list, code generation | Unit | Stable |
| Admin: Assignments | `src/admin/assignments.rs` | Server fns + types only: generate, swap, get_preview | E2E | Stable |
| Admin: SMS | `src/admin/sms.rs` | Server fns only: send_* (4 SMS types), SmsReport, AssignmentTarget | E2E | Stable |
| Admin: DB Helpers | `src/admin/db_helpers.rs` | ActiveSeasonRow struct; fetch_active_launched_season() shared predicate (SSR-only) | Unit (via callers) | Stable |
| Assignment | `src/assignment.rs` | Assignment algorithm: cycle validity, social-weight minimization, cohort splitting | Unit | Stable |
| Error | `src/error.rs` | strip_server_error_prefix() — strips framework prefix from ServerFnError display | — | Stable |
| Hooks | `src/hooks.rs` | use_hydrated() -> ReadSignal<bool> — false during SSR/hydration gap, true after WASM init | — | Stable |
| I18n | `src/i18n.rs` | load_locales!() macro; i18n context available to all components | — | Stable |
| SMS | `src/sms.rs` | TurboSMS HTTP client (SSR-only; dry-run via SAMETE_SMS_DRY_RUN) | — | Stable |
| Date Format | `src/date_format.rs` | format_date_uk() — UTC OffsetDateTime → Ukrainian locale string | — | Stable |
| Pages | `src/pages.rs` | pub mod re-exports for home, login, onboarding | — | Stable |
| DB | `src/db.rs` | Pool creation, migrations | — | Stable |
| Config | `src/config.rs` | Env-based config | — | Stable |
| Types | `src/types.rs` | Domain types, enums | — | Stable |
| Phone | `src/phone.rs` | E.164 normalization | Unit | Stable |

Deleted in 2026-06-20: `src/admin/dashboard.rs` (DashboardPage — dead), `src/admin/nav.rs` (AdminNav — dead). Dead components stripped from season.rs, assignments.rs, sms.rs — only server functions retained.

Changes in 2026-06-22: Component system elevation — 43 fixes across CSS + 5 Rust files. A2 tokenization (141→60 bare values). Admin timestamps pre-formatted as String (no hydration mismatch). Dead `ParticipantList`/`ParticipantsPage` deleted from participants.rs. 4 SMS button i18n keys added. `data-layout="admin"` moved outside Suspense for SSR density. All submit buttons have `aria-busy`. All form errors wired with `aria-invalid`/`aria-describedby`/`aria-live`.

## E2E Test Suite

- **Total:** 119 tests (75 mail_club.spec.ts + 43 visual-audit.spec.ts + 1 visual-audit-cohort.spec.ts; 2 by-design skips in full mode)
- **Pass rate:** 117/119 — verified pre-push T-CAP session (mode=full on isolated harness)
- **Runtime:** ~55s (release, local); ~2m (dev, local — flaky, see deferred items)
- **Structure:** mail_club.spec.ts — main lifecycle chain + Account Management + Session Management. visual-audit.spec.ts — 43 capture tests across 4 mode dirs (light-desktop, light-mobile, dark-desktop, dark-mobile) + sections/
- **Pipeline:** `just e2e` → `just e2e-release` (release binary). `just e2e-dev` for dev mode. `just capture-isolated <suffix> [visual|full]` for isolated capture.
- **Fixture:** `cached-context.ts` caches WASM/JS/CSS/fonts across tests. `capture-constants.ts` exports shared viewport/timing/phone constants.
- **Pre-compression:** `precompress-and-test.sh` runs before every E2E
- **Wait strategy:** Zero `waitForLoadState` calls. URL assertions for redirects, element assertions for interactivity. `confirmReady`/`confirmReceipt` have explicit `toBeEnabled()` guards. `goHome`/`goToDashboard` use `<main>` visible (NOT hydration wait — that causes 30s hangs).
- **Screenshots:** `end2end/screenshots/{light-desktop,light-mobile,dark-desktop,dark-mobile,sections}/` — gitignored; `INDEX.md` manifest written by visual-audit afterAll

## WASM

- Dev: 14MB raw, 1.9MB brotli (pre-compressed)
- Release: 1.87MB raw, 471KB brotli
- Profile: `opt-level = 'z'`, `lto = true`, `codegen-units = 1`
- wasm-opt: automatic via cargo-leptos 0.3.2 (`-Oz`)
- Optimization floor reached. Remaining gains: `build-std` (nightly) or `--split` (code splitting).

## Infrastructure

- Postgres 16 in Docker (`docker-compose.yml`)
- cargo-leptos 0.3.2
- Leptos 0.8.17, Axum 0.8.8
- macOS linker: classic (`-Wl,-ld_classic`) due to Apple ld assertion bug with thin LTO
- GitHub Actions CI: `.github/workflows/ci.yml` — `check` job + `e2e` job with Postgres service

## Known Limitations

1. Leptos SSR has no per-Resource timeout (router-level middleware was incompatible with Suspense streaming)
2. `leptos_config` pulls `regex` into WASM dependency tree (LTO eliminates it, but compilation is slower)
3. Docker Postgres adds latency vs native
4. `CompressionLayer` re-compresses SSR HTML on the fly (small, fast — not a bottleneck)

## Changes 2026-06-25 (visual-immaculate campaign — ~44 fix commits on main, HEAD `bb078b2`, NOT pushed)

Verified premise correction: the component system is INTACT (the "no component system" report was false). Defects were rendered-pixel issues, fixed across 5+ re-verify rounds. Non-admin areas (auth/onboarding/home A/B) verified materially improved. **Admin still janky** (see below + deferred_items) — campaign paused there.

**Toast** (`src/components/toast.rs` + `.toast-container`): now a **sticky flow-banner inside `<main>`** (`position: sticky; top:0; width:100%`, mounted before `<Routes>` in app.rs) — pushes content down, never overlaps. Was a fixed overlay; 3 fixed-position attempts collided with cycle-graph / desktop gutter / header / home h1. Slide-down keyframe; `data-testid="toast"` preserved.

**Heading hierarchy:** home state headings promoted `<h2>`→`<h1>` (page-title clamp scale); admin section headings sized via `[data-layout="admin"] --text-section: 1.1rem`; `.prose-page h1` letter-spacing `-0.01em` (Cyrillic glyph overlap at clamp ceiling); `h1/h2/h3.overline-label` specificity rule so headings can carry the overline style; the `[data-layout="admin"]` density block wrapped in `@layer base` (was unlayered CSS).

**`.data-table`:** `td:last-child { min-width:7rem; white-space:nowrap }`; `td vertical-align: top` (wrapped cells align top). Invite-codes table now `.data-table--invite` (class on the `<table>` in admin/page.rs) with `table-layout: fixed` + explicit column widths — ⚠️ **CURRENTLY OVERPRINTS** (status badge over redeemer, code over distributor on used rows): 5 cols don't fit 65ch. TO BE REPLACED by a form-first fix (cards / fewer columns / break 65ch). Mobile card-stack degradation intact + clean.

**New:** `src/components/skeleton.rs` (`SkeletonFallback`, replaced 3 inline skeleton blocks; `components/mod.rs` updated). `guidance/ui-review-prompt.md` (shared UI reviewer method).

**Dead classes removed** (-70 lines CSS): `.stat-card/.stat-value/.stat-label/.action-panel/.action-panel-title/.danger-zone/.danger-zone-title` (zero Rust usage).

**ARIA:** home enrollment inputs + admin swap selects got `aria-invalid` + `aria-describedby="action-error"`.

**Rust:** `strip_server_error_prefix` made `pub(crate)` (login.rs), reused in onboarding.rs error display (was duplicating the framework prefix). `enroll_in_season` gained `use_existing_address: bool` param (explicit skip; replaced implicit empty-string contract; form sends it as a hidden input in both branches).

**Infra/tests:** `justfile db-reset` prepends `pg_terminate_backend` (external SQL clients block the drop). `end2end/tests/visual-audit.spec.ts`: merged the 23/24 duplicate capture, renamed 29→`home-cancelled-season`, 03→`admin-invite-codes-initial`, 09→`admin-no-season-create-form-available`. Screenshot set now **28 page-pairs** (was 29). Mobile phase-stepper fit attempt still clips steps 1/5 (deferred — needs compact "step N/5" form).

**Recon artifacts:** `orchestration_log/recon/2026-06-24/` — `adhoc-audit-*`, `reviews/` (spec-/quality-/review-*), `reverify-*` (R1–R5 + def/revoke/final/gate), `fix-plan.md`, `fix-plan-round2.md`, `css-regression-rootcause.md`, `toast-rootcause.md`, `admin-revoke-investigation-2.md`, `e2e-failure-investigation.md`, `db-blocker-fix.md`, `screenshot-staleness.md`, `recent-sessions-digest.md`.

**Worktrees:** all session worktrees + 3 stale prior-session worktrees removed; only main worktree remains.

## Changes 2026-06-25 (continued — sectioned screenshots on main; admin de-jank in worktree)

- **On main (`85f9ccb`):** `visual-audit.spec.ts` now captures per-section admin screenshots (`screenshots/sections/{NN}-{state}__{section}.png`) in addition to full-page; 11 section `data-testid`s added to `src/admin/page.rs`. Screenshot set: 28 desktop + 28 mobile full-page + 102 admin section crops. `captureSection` helper; cancel-confirmation dialog captured.
- **SHIPPED to main + pushed (origin/main @ `405b955`, **CI green** run 28172280159) — supersedes the "NOT integrated" note below:** invite-codes table → card form (`<ul class="invite-code-list">`/`<li class="invite-code-card">`, all viewports; `.data-table--invite` table-layout:fixed CSS removed); button-group `items-start`; participant-count space; cancelled-stepper `data-status="abandoned"` treatment; compact mobile phase-stepper (`"Крок N з 5: <label>"` ≤640px, full strip ≥640px); even invite-card `min-height`; 2 `locales/uk.json` keys. Files: `src/admin/page.rs`, `src/components/stepper.rs`, `style/tailwind.css`, `locales/uk.json`.

## Changes 2026-07-03 (complete visual review — NO code changes)

- **Review-only session. No source or CSS changed.** HEAD still `5ba82ec` (= origin/main, doc-only over `405b955`). Working tree clean.
- **Component-system verdict (authoritative — 36-agent review + static inventory):** primitives are SOUND. One `.btn` (every button traces to it via `data-variant`/`data-size`), one `.field`/`.field-input`, one `.badge[data-status]`, single palette (contrast-corrected orange `oklch(0.63 0.22 31)`, no `#FB4417` anywhere), single 65ch `.prose-page` column. Confirmed intact. **The jank is SEMANTIC not structural:** correct components bound to wrong meanings — badge tokens shared across conflicting functions, primary actions styled `secondary`, typography levels (h3, auth h1) applied by accident.
- **Dead CSS confirmed (inventory):** `.badge[data-status="pending"|"error"]`, `.deadline[data-urgency="soon"|"imminent"]` (+ `.deadline[data-urgency="imminent"] .deadline-value`), `.deadline-label`, `.deadline-value` = 7 rules, zero `src/` usage. `.deadline` itself is used as a flat `<p>` — its multi-element flex label/value structure is never emitted (see deferred S6).
- Full defect backlog + evidence: `recon/2026-07-03/DEFECT-CATALOG.md` + `pages/*.md` (×28) + `concerns/*.md` (×8) + `inventory/component-instances.md`.
- **Screenshot set current** as of `405b955`: 28 desktop + 28 mobile + 102 admin section crops (`end2end/screenshots/{desktop,mobile,sections}/`, gitignored). Dark-mode + full-cohort cycle-viz remain uncaptured (deferred).

## Changes 2026-07-04 (visual-fix campaign — main @ 7795997, UNPUSHED)

~10 fix commits (76de205..7795997), all spec+quality reviewed+integrated; e2e green 110/0; 8 blockers CLEARED in rendered pixels.

- **Dark mode NOW RENDERS** (was spec'd in tokens, never rendered → 7 blockers): new `--color-border` semantic alias (light=brand-gray / dark=oklch(0.58)); dark `--color-surface-raised` 0.18→0.22; dark `--color-error`/`--color-success` → 0.68/0.72; `.alert`/`.sms-report-result` → solid `--color-panel-dark` + accent stripe; dark-only `--color-panel-dark`/`--color-step-idle` (with base-:root light fallbacks); cream dark select-arrow; secondary-btn border pinned in dark. design-system.md §Dark Mode/§Palette/§Badges synced (brand-gray hex → #51565B; badge table matches shipped CSS).
- **Error affordances fixed app-wide:** `attr:aria-invalid`→bare `aria-invalid` at all 13 inputs (grep=0); error copy localized (login 6 + admin create-season 4 keys); admin action-error banner + login sites use `strip_server_error_prefix`.
- **Other:** S1 revoked-badge→gray + dead pending/error variants removed; S2 deactivated actions-column→muted em-dash (state via STATUS badge only); S3 advance/swap→primary variant (dominant over destructive); S4 auth `<h1>` on 4 steps + logo h-20→h-32; S5 `.prose-page h3` + lone overline-label→plain h2; S6 `.deadline` simplified-to-flat (dead child/urgency rules removed); S7 `.btn-group` primitive + tag-agnostic rhythm (`>` direct-child retained + widened list); overflow (invite-card redeemer name/date split + even heights, participant name-cell wrap, theme word-break + maxlength=100).
- **Capture tooling (visual-audit.spec.ts):** dark (`emulateMedia`, 28 states ×2 → dark-desktop/dark-mobile), error (3 forms `__error`), focus (`__focus`), long-content. NN screenshot prefixes renumbered by the error/focus insertions (state NAMES stable).
- **2 residuals + tail open** (see deferred_items). Push HELD.

## Changes 2026-07-04 (fix-everything Wave 1 — on branch, NOT yet on main)

main still @ 7795997. Wave-1 CSS foundation on branch fix/css-wave2 @ ae39300 (/private/tmp/csswave-wt), spec PASS + quality pending, to ff-merge on quality-green:
- **New mode-invariant `--color-badge-*` token system** (in `@theme`) + every `data-status` remapped: active/confirmed/complete→green(0.50), ready→muted-blue(0.70), unused/pending→amber(0.82), inactive/used/revoked→brand-gray, error→dedicated red(0.55). All AA in BOTH modes (fills don't reassign). Broke the unused/confirmed blue collision; fixed a real `active` AA fail. `design-system.md §Badges` rewritten to this system.
- `.admin-section` raised-surface card class (S11 — markup wrap pending in Wave 2). `.prose-page dl:not(.info-list)` scope (L5). stepper.rs `is_complete` → all-steps-completed-green + cancelled visible at all widths (S14).
- Wave 2 (admin markup + i18n + home + onboarding + login + toast.rs S9 + RV-3/V4) QUEUED, not started (session stopped after Wave 1 per user).

## Update 2026-07-04 (Wave 1 now ON main)

Wave-1 CSS foundation INTEGRATED: main @ ae39300 (was 7795997). The mode-invariant `--color-badge-*` status-color system, `.admin-section` raised-surface card class, `.prose-page dl:not(.info-list)` (L5), and stepper terminal-state logic (S14) are now on main + design-system.md §Badges rewritten. UNPUSHED. Wave 2 (admin markup + i18n + home/onboarding/login markup + toast.rs S9 + RV-3/V4) queued, not started (session stopped after Wave 1 per user).

## Changes 2026-07-04 (Wave 2 visual fixes — session 7c7c3839)

main @ **d4a9396** (17 commits ahead of ae39300; UNPUSHED at session close; push authorized pending CI-preflight). Session branch: `visual` (worktree css-systemic-fixes).

**Wave 2 shipped: 7 fix-units + 2 rendered-re-verify-caught fixes + 1 E2E capture** (ae39300→d4a9396). All through independent spec→quality→integrate-on-BOTH-green. 3× E2E green (release, 110/0).

**Per-file changes:**
- `src/admin/page.rs` — `.admin-section` raised-surface cards (4 peer cards, no nesting); SMS report→`.badge` co-located with each trigger; cancel hierarchy (initiate=secondary, confirm=destructive); pre-launch participant count uses distinct `admin_pre_launch_participant_count` key; overline labels on invite sub-sections + real `<label for="invite-code-filter">`. `aria-invalid` bare (no `attr:` prefix) on all inputs.
- `src/admin/{season,invite_codes,assignments,sms}.rs` — ~17 user-facing server errors → `td_string!(Locale::uk, ...)`. Infra/500-class left English. All admin errors flow through `action_error` closure → `strip_server_error_prefix`.
- `src/pages/home.rs` — enroll + confirm-ready CTAs `w-full`; saved recipient address rendered as labeled block (`ВІДДІЛЕННЯ`/`ТЕЛЕФОН` overline rows); no-season state uses `.empty-state` with `<h1>`. Receipt toast trigger removed (suppressed on confirmation-destination states). i18n: existing-address value via `home_recipient_branch` key.
- `src/pages/onboarding.rs` — per-field aria error routing: `RejectedField` enum + `rejected_field_from_error` classifier; each field has its own `aria-describedby` target; form centering scoped (doesn't collapse column width). `#[allow(clippy::too_many_lines)]` added (view! verbosity, justified). `np-number-error` testid added. Redundant clone dropped; `None` field = neither-field (infra errors).
- `src/pages/login.rs` — OTP input capped `max-w-[12ch]` on `.field-input` (not `.field`) so error text renders full-width. Back-button top margin via `mt-(--density-space-sm)` token (not bare `mt-3`; `.btn + .btn` can't match across `<form>` boundary).
- `src/components/toast.rs` — auto-dismiss ~4s (`set_timeout_with_handle` + `StoredValue<Option<TimeoutHandle>>` + `on_cleanup`); named timing constants (dismiss_ms/exit_ms; unsigned subtraction compile-safe); uniform `try_get_value`; `data-state="leaving"` drives slide-out animation.
- `style/tailwind.css` — `.field-error { text-align: start }`; `.field-input::placeholder` explicit muted color (measured 7.34:1 light / ~5:1 dark); `.invite-code-card { min-height: ... }` for even heights; dead `.toast[data-type="error"]` branch removed; `@media (max-width:639px)` stacks invite-code cards full-width (matching `.data-table` breakpoint); `@keyframes toast-out` + `.toast[data-state="leaving"]` + reduced-motion suppression.
- `guidance/design-system.md` — disambiguated badge-`pending` (amber, open-invite) vs stepper-connector-`pending` (muted dot between unvisited steps); `.admin-section` forward-prep note; `confirmed`→green reclassification note.
- `locales/uk.json` — `admin_pre_launch_participant_count` key; `admin_invite_codes_filter_label` key; ~17 admin error i18n keys; `dashboard_enrolled_label` orphan removed; `home_recipient_branch` reused for existing-address.
- `end2end/tests/visual-audit.spec.ts` — new `captureElementState` for `11-onboarding-branch-selection__error` (per-field validation error; `clickAndWaitForResponse(...,"complete_onboarding")`, non-persisting reject, serial-safe).

**Screenshot infra (unchanged structure, +1 error-state capture):**
- 33 desktop, 33 mobile, 28 dark-desktop, 28 dark-mobile, 102 admin section crops

**Rendered re-verify verdicts (main @ 094ca1a, then c3433f5):**
- rv-admin: 7 Wave-2 fixes landed in pixels. ONE MAJOR caught: invite-card mobile balloon → fix c3433f5. Admin CLEAN after fix.
- rv-participant: OTP error-text 4-line wrap MAJOR caught → fix 0422a77. Auth/onboarding/home otherwise CLEAN.
- rv-dark: CLEAN (0 defects). Badge, placeholder, `.admin-section`, toast, stepper all verified in dark. Coverage gap: `field-error` in dark (no error-state in 28×2 dark set — RV-3 in dark remains uncaptured).
- rv-fixes, rv-onboarding-err: both CLEAN.

**Key root causes fixed this wave:**
- `.btn + .btn` can't match across `<form>` → use token-based margin, not adjacent-sibling combinator
- Width cap on `.field` (not `.field-input`) lets error text inherit the cap → cap the input element, let the field wrapper stay full-width
- Desktop left/right card split retained on mobile without explicit breakpoint override → `@media (max-width:639px)` stacking rule required

- **2026-07-04 close:** main @ `77d7d05` (code d4a9396 + close-docs c6bae49 + conventions-dedup 77d7d05), pushed to origin/main. CI run 28717592269 GREEN (Check + E2E). Wave 2 complete.

- **2026-07-05 follow-up:** onboarding participant errors localized via language-independent field routing (c487247+f3291e5) + 8 orphan uk.json keys removed. main @ f3291e5, CI run 28719895709 GREEN (Check + E2E). Resolves the onboarding-i18n + orphan-keys deferrals.

## Changes 2026-07-09/10 (visual design-intervention campaign — checkpoint, IN PROGRESS)

main @ **17e9891**, UNPUSHED, clean tree. Six fault-threads integrated via full spec→quality→rendered-verify→integrate chains; one (auth) blocked.

- **Isolated capture harness (NEW, on main):** `scripts/isolated-capture.sh`, `end2end/playwright.config.ts` `CAPTURE_BASE_URL` env fallback (line ~30), `capture-isolated` justfile recipe. Runs any worktree binary on a free port + sibling `samete_<suffix>` DB on the live Docker Postgres, playwright via CAPTURE_BASE_URL, trap teardown, `!=3000`/`!=samete` guards. Bypasses `cargo leptos end-to-end`. Enables parallel captures that never touch the user's :3000/samete. Invocation: `cd <worktree> && bash scripts/isolated-capture.sh <suffix> [visual|full]`.
- **`.page-frame` layout primitive (NEW, on main):** viewport-height (`min-height: calc(100dvh - var(--header-height))`); short/terminal participant states center via `.page-frame > .prose-page > .empty-state { margin:auto }` (`.empty-state` is a grandchild of `.page-frame` — `.prose-page` between; Suspense emits no wrapper). Scoped to participant home ONLY (never wraps admin). `--size-logo-height-header` token. `.empty-state .empty-state-headline` (0,2,0) out-specifies `.prose-page h1` for --text-section scale.
- **Header (app.rs shared Header fn):** rebuilt — dominant mark, logout=quiet pill, admin-nav=`.header-nav-link` text link. Shared across participant/admin/login/onboarding; participant-home layout fix did NOT touch shared `<main>`.
- **home.rs participant states:** 7 short states route through `.empty-state` (centered); Cancelled/NoSeason/Confirmed/Assigning/ReceiptConfirmed/Complete/Enrolled. Preparing SPLIT: deadline_passed=true (announcement)→centered empty-state; deadline_passed=false (confirm CTA)→top-flow. EnrollmentOpen: `.info-list` address block + deadline constraint sentence (LONG, top-flow). Enrolled: zero-CTA empty-state wait card.
- **admin/page.rs Season Management:** typography consolidated; status→aligned `.badge`; per-phase IA 3-way gate on `season.launched`; unified counter/metric treatment; `.sms-report-result` margin; 4 SMS buttons→secondary, "Далі" sole primary.
- **BLOCKED:** auth (login.rs) redesign + OTP resend in worktree agent-ad20d8ab @ bd7c4d9 — `/login` SSR abort unresolved (see deferred_items).

## Changes 2026-07-10 (auth integration + T08 + #10)

main @ **443efdd**, UNPUSHED, clean tree (except orchestration_log docs). No CI run yet.
- **Auth (T02+T03) INTEGRATED** (5da012b, 13 rebased commits): `.auth-card` + `.btn[data-variant="link"]` + page-frame wrap + OTP resend affordance (RwSignal cooldown, dual cfg'd `resend` bindings — client dispatch+cooldown / SSR no-op — OUTSIDE view!; interval_handle StoredValue + on_cleanup cfg-gated outside view!). login.rs:~888 braces on `disabled={...}` are LOAD-BEARING (bare `>` eats the tag). visual-audit.spec.ts: clearCookies before /login captures + toBeVisible step-gate + toBeEnabled hydration wait.
- **isolated-capture.sh** (47226e8): playwright exit propagated through EXIT-trap teardown; screenshot floor (exit 1 on exit-0-with-0-pngs); marker cleaned in trap. Exit code now TRUSTWORTHY (supersedes "gate on screenshot count only" workaround).
- **cycle-viz** (443efdd): render_cycle_ring viewBox `0 0 600 560`, center(300,260), named geometry constants (RING_RADIUS/CENTER_X/CENTER_Y/NODE_RADIUS/LABEL_OFFSET_Y); `.cycle-viz` aspect-ratio 600/560 + `--size-cycle-viz: 480px`.
- Screenshot sets in worktrees are GONE (worktrees removed); regenerate via `bash scripts/isolated-capture.sh <suffix> visual` from any tree.

## Changes 2026-07-11 (T-CAP: capture-suite elevation — main @ d6a17f6, PUSHED, CI 29132991848 green)

18 commits over d06b4ba, full spec→quality→pixels chain. Screenshot capture is now the authoritative review artifact:
- **`end2end/tests/visual-audit.spec.ts` rewritten:** stable `{area}-{state-slug}[__{variant}].png` naming (no execution-order coupling — filenames survive partial runs, diffable across runs); dirs `light-desktop|light-mobile|dark-desktop|dark-mobile` (+`sections/`, pruned 102→3 crops); atomic `recordScreenshot` primitive (sole `page.screenshot` site) appends MANIFEST → `screenshots/INDEX.md` (file|stateId|route) written in afterAll; beforeAll stale-png cleanup; error/focus variants captured in dark too; Pass B (revoked code, deactivated participant) + Pass C (existing-address enrollment, pre-submit not-received form `H7b`); no-season capture guarded via `season-cancelled` probe (honest in full-suite context). 43 tests.
- **`end2end/tests/visual-audit-cohort.spec.ts` (NEW) + `end2end/tests/fixtures/cohort-seed.sql` (NEW):** 12-node cycle-viz pass, direct SQL seed (12 double-barrel-Ukrainian participants, ring assignments) into the harness sibling DB. Triple-guarded: file-scope `test.skip(COHORT_CAPTURE!=="1")` (blocks beforeAll under default discovery), `assertSiblingDatabaseUrl` (throws on unset/`samete`), harness exports the flag only for cohort VISUAL_SPEC. Seed + INDEX-row merge both idempotent. Invocation: `VISUAL_SPEC=tests/visual-audit-cohort.spec.ts bash scripts/isolated-capture.sh <suffix> visual`.
- **`scripts/isolated-capture.sh`:** `"${VISUAL_SPEC:-tests/visual-audit.spec.ts}"` override + cohort case-block. `mode=full` + isolated port/DB = CI-way full-suite validation without touching :3000/samete (117/119 passed / 2 by-design skips verified pre-push).
- **5 designed captures removed as unreachable-by-design** (anti-enumeration silent-Ok paths, HTML5 required gating, no-op same-participant swap, always-seeded resend cooldown) — each with source-cited WHY comment. Gap-analysis lesson: DOM-element presence ≠ reachable state; trace server semantics.
- **Artifact contract:** 174 pngs == 174 INDEX rows (two-direction verified); INDEX.md is the navigation manifest for review.
- App-defect catalog (7 actionable + 1 design question) extracted to `orchestration_log/history/2026-07-09/app-defect-catalog.md`.

## Observed 2026-07-12 (sustainability discovery — no code changes yet)

13-concern audit sweep (recon/2026-07-12/audits/, contract-constrained): ALL 13 FAIL — 2 BLOCKER, ~30 MAJOR, distilled to 30 fix-units in recon/2026-07-12/SUSTAINABILITY-BACKLOG.md (bound re-derivation; 14 weak findings rejected). Headlines, cross-corroborated: `just prepare` recipe runs the forbidden bare `cargo sqlx prepare` (wipes offline cache — the documented footgun is IN the tool); 41/50 unit tests invisible to bare `cargo test` (pure modules gated behind `#[cfg(feature="ssr")]`); phase semantics duplicated typed-Rust vs raw-SQL with one silent predicate drift (sms.rs); participant-facing English via `e.to_string()` in home.rs flows; untyped phone across the auth boundary; N+1 SMS inserts ×4 fns; missing OTP/user indexes; untransacted 2-statement deactivate; silent stale-address enrollment on partial input; dead csrf_secret config; unused blake2 dep; tailwind.css 1626 lines with fragmented .btn/.field-input blocks + dead modal system; zero geometric E2E assertions; `attr:aria-current` literal-attribute leak in stepper.rs. Execution campaign starts Wave 1 (10 parallel QUICK-WINs + 2 sequential justfile units).

## Changes 2026-07-15 (visual-fault campaign — main @ 59bef87, UNPUSHED, fix-loop COMPLETE / re-verify+preflight PENDING)

34 commits over `77b2072`. **All 27 fix-units from `recon/2026-07-15/FIX-UNIT-CATALOG.md` integrated** (3 BLOCKER / 19 MAJOR / 5 MINOR) + focus-ring (deferred-IN) + grain doc-sync. Every unit through implement(worktree)→spec-review→quality-review→integrate-on-both-green (56 verdict files in `recon/2026-07-15/reviews/`). FU-16 verified no-op. **NOT YET DONE: final holistic rendered re-verify, CI-green preflight, push (HELD).**

Discovery/planning artifacts (recon/2026-07-15/, gitignored): `REVIEW-CONTRACT.md`, `inventory/component-inventory.md`, `reviews/axisA-*.md ×8` + `axisB-*.md ×8`, `FIX-UNIT-CATALOG.md`, `design-direction.md`.

**Per-area code changes (all on main):**
- **CSS foundation** (`style/components.css`, `style/tokens.css`): `.auth-card { width:100% }` so max-width binds to 28rem under flex auto-margins (FU-01); destructive button → mode-invariant `--color-badge-error` (FU-06, +design-system.md doc-sync); `--text-secondary` → 0.875rem + 7 inline call-sites routed through it (FU-07); grain overlay dropped `mix-blend-mode:overlay` → normal blend renders on all surfaces (FU-08); `--header-height` collapsed to one token consumed by token/`.app-header`/`.page-frame` (FU-09); light `--color-focus` → `oklch(0.58 0.15 250)` = 4.04:1, dark keeps brand-blue via explicit override (focus-ring, WCAG 1.4.11).
- **Canonical error-region (FU-04)** (`error.rs`, `admin/season.rs`, `admin/page.rs`, `pages/onboarding.rs`, `pages/home.rs`, `uk.json`): one `.field-error` shape everywhere a field-specific error renders; `enroll_in_season` now discriminates city vs np_number; `FIELD_DISCRIMINANT_SEPARATOR` consolidated to `error.rs` (single def + WHY doc-comment); 4 raw `e.to_string()` sites routed through `strip_server_error_prefix`. Admin create-season deliberately shows field-error in BOTH local + banner to preserve E2E assertion (visual-audit.spec.ts:514).
- **home.rs**: new `HomeState::EnrollmentNotOpen` distinct from NoSeason (FU-03, reachable-by-construction); saved-address enrollment framed in `.card` (FU-19); enrolled-state deadline unified to `.deadline` (FU-13).
- **onboarding.rs**: adopts shared `.page-frame`/`.auth-card` shell + dark logo swap + h1 section-scale; dropped `pt-[10svh]` hack (FU-02). No back-button (no valid back-target — owned YAGNI call).
- **login.rs**: auth hero logo dark-swap → `/logo-white.svg` via `use_preferred_dark` (FU-15). FU-16 (link-variant weight) already consistent, no-op.
- **app.rs**: 404 CTA → primary (FU-14); theme-color↔token WHY comment (FU-27); **mobile drawer DELETED** — `MobileMenu` component, `menu_open` signal, hamburger, overlay, `slide-in-right` keyframe, `--size-mobile-menu` token, `nav_home` key all removed; `HeaderNav` (`.header-nav-link`) renders inline at all viewports (FU-22).
- **admin/page.rs**: canonical create-season card unbraiding + date-label disambiguation (FU-10); short filter placeholder (FU-17); uniform Учасники h2 (FU-18, subsumed by FU-21's group-label-section removal); **admin brand/hierarchy** — CyGrotesk h1 anchor (`--text-page-title 1.75rem` admin), `--size-admin-content-max: 64rem` breaks the 65ch cap, stepped headings (FU-21); **button emphasis graded** — one orange primary per phase, destructive-red only on cancel-confirm (FU-20); `.sms-trigger-action` stacks SMS rows on mobile (FU-11); invite-code preview → `.invite-code-card-code` span (drop `<code>`-preflight monospace) (FU-25).
- **cycle-viz (FU-12)** (`admin/page.rs` render_cycle_ring + `.cycle-viz`): ring radius + viewBox scale to cohort size (n≥12 reproduces prior 900×750/190 geometry exactly); `ring_radius_for_cohort_size` + 6 boundary unit tests (suite 62→68).
- **stepper (FU-23)**: every connector status has an explicit rule via `--color-step-idle` (zero-visual-diff clarity refactor).
- **toast/alert (FU-05)** (`components/toast.rs`): unified into one feedback register (shared shape, status-color-differentiated); light toast text uses mode-invariant `--color-badge-success` (AA); dead `.toast-message`/`--toast-border` removed.
- **density (FU-26)**: 3 ad-hoc gaps → `--density-space-*` (skeleton.rs, admin/page.rs, login.rs; the login one removed an inline `mb-4` shadowing an existing token-routed CSS rule).
- **empty-state headlines (FU-24)**: unified to no-terminal-punctuation (uk.json, 2 outliers).
- **Docs synced**: design-system.md (destructive token, focus token, grain), frontend-protocol.md (grain). `visual-audit.spec.ts`: G7 mobile-menu capture test removed (state deleted).

**Rendered-verify status:** CSS foundation was rendered-verified (grain/destructive/helper/header CLEAR; FU-01 residual caught+fixed in pixels). The other 26 units passed CODE review only — a FULL holistic rendered re-verify (both viewports, both modes) is the outstanding trust gate before this campaign is complete.

**Verification-orchestration map** (per-unit screen + pixel-check + rendered-verify status for task #4, priority order, do-not-re-flag list, capture invocations): `recon/2026-07-15/VERIFICATION-HANDOFF.md` (gitignored/disposable — the offloaded per-fault detail for high-fidelity re-verify).

## Changes 2026-07-16/17 (visual re-verify + residual-fix campaign — IN PROGRESS, UNPUSHED)

main @ **fc934d1**, UNPUSHED (ahead ~52 of origin/main), clean tree. Continuation of the 2026-07-15 27-unit campaign: a full rendered re-verify surfaced residuals, fixed in two rounds.

- **Round-1 residual fixes INTEGRATED (14 units + wordmark, commits 2766173..fc934d1):** RV-01 dark-toast cascade order + mobile CTA safe-area padding; RV-02 auth-card mobile bezel inset; wordmark hero swap logo.svg->icon-only same_te_mark_*.svg (`--size-auth-logo` 5rem->3.5rem in tokens.css); RV-10 onboarding heading centering matches login (reuses `.auth-card > div`); RV-05 all label/value meta routed through canonical `.info-list` (shared-label-column grid); RV-08 cancelled-season compact mobile stepper (+ dead `.stepper-compact[data-status=abandoned]` removed); RV-06 cycle-viz geometry scales to cohort size (`CycleGeometry` struct, `scale_for_cohort_size`, 12-node unregressed test); RV-07 receipt-confirmed copy branches on `ReceiptStatus` (`HomeState::ReceiptConfirmed` now carries status); RV-12 static amber pending badge on Assigning; RV-03 admin create-form heading->overline; RV-04 admin create-season error single-surface (signals Option<String>->bool, aria only, banner sole surface); RV-11 pre-generate card description; RV-13 `CreateFormContext` enum suppresses redundant no-season line; RV-14 dark header border pinned to full-strength `--color-border`.
- **Both round-1 blockers confirmed fixed in fresh pixels** (RV-01 dark-toast AA, RV-14 header border).
- **Round-2 residuals QUEUED (not yet fixed):** `recon/2026-07-15/RESIDUAL-CATALOG-2.md` = 12 units (2 blocker / 7 major / 3 minor). Blockers: R2-01 CyGrotesk renders Cyrillic `є` as `с` (font-glyph, latent app-wide incl home h1; needs root-cause: unicode-range fallback vs corrected asset vs new-asset), R2-02 12-node cycle-viz labels illegible (regression of RV-06). Majors: `.info-list` canonicalization incomplete (admin season-meta, invite redeemer, saved-address); two admin action-subgroup grammars; semantic-color misuse (error-red for a routine stat, deadline-callout border varies by state).
- **Screenshots refreshed** at fc934d1: `end2end/screenshots/{light,dark}-{desktop,mobile}/` = 42 states/dir (41 main + `admin-cycle-viz-12-nodes` cohort) + INDEX.md. Gitignored.
- **Guidance docs reconciled** (a152b39 + earlier): design-system.md, frontend-protocol.md, component-evaluation-framework.md synced to shipped behavior (grain blend, z-index, admin width, destructive-button table, auth-hero asset row).
