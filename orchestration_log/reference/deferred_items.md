Flushed 2026-06-20. Nothing deferred.

## Added 2026-06-22

### Dev-mode WASM hydration intermittent failure
- **Severity:** Medium (mitigated by release-binary E2E, but dev-mode debugging harder)
- **Detail:** 14MB dev WASM intermittently fails to call `hydrate()`. Buttons stay disabled permanently. Affects ~40% of dev-mode E2E runs. No server errors, no WASM fetch failures — client-side init failure. Investigated: cached-context encoding, POM waits, goHome hydration probes, reload fallbacks. None solved it.
- **Mitigation:** `just e2e` now runs release binary (471KB WASM). `just e2e-dev` preserved for manual debugging.
- **Rationale for deferral:** Release binary is stable (3/3 greens). Root-causing the dev WASM init failure requires Leptos/wasm-bindgen level investigation beyond component work scope.

## Added 2026-06-24

### Unreviewed commits on main
- **Severity:** High (process violation)
- **Detail:** 5 commits integrated without spec/quality review chain: b2a88cd, 8bdd7a1, ddac0bd, 543bbbc, 72d029a. Covers TurboSMS field fix, error prefix strip, onboarding redirect (2 commits), enrollment address form + POM.
- **Action:** Run post-hoc review chain on diff range b2a88cd~1..72d029a before next push.

### CSS layout appears broken despite classes being present
- **Severity:** High (user-facing)
- **Detail:** User reports layouts broken everywhere. CSS investigation confirmed all 33 component classes compile and serve at correct file. Deep investigation (checking whether classes are applied to HTML elements) was in progress when session ended.
- **Action:** Complete the deep investigation. Check SSR HTML output for class application. Check if Rust components actually use the component classes vs raw utilities.

### E2E suite not re-run after enrollment fix
- **Severity:** Medium
- **Detail:** Commits 543bbbc (address form) and 72d029a (POM update) not verified with E2E. 3 consecutive greens needed.
- **Action:** Run `just e2e` 3x before pushing.

### IP-based rate limiting absent
- **Severity:** Low (per-phone limits exist)
- **Detail:** No IP-based rate limiting on OTP requests. Per-phone limits (1/60s, 5/hr) mitigate but don't prevent distributed attacks across phones.
- **Rationale for deferral:** Architectural — needs middleware or external store. Different category from the quick fixes addressed this session.

## Added 2026-06-25 (visual-immaculate campaign)

### RESOLVED this session (from 2026-06-24 list)
- **Unreviewed commits on main** — review-debt audit run on `11c55ed..72d029a`; 2 Important fixed (onboarding error prefix `a3f6ca5`, address-skip explicit flag `47984a7`). Cleared.
- **"CSS layout broken / no component system"** — DEBUNKED. Component audit (buttons 31/31, fields 51/51, badges 6/6 canonical) + CSS root-cause (built CSS correct, 33 classes in `@layer components`) prove the system is intact. The real defects were rendered-pixel issues (typography, toast, table), fixed across the campaign. Was a false inherited belief from the prior blocked session.
- **E2E not re-run** — run ~9 times this session; release suite green (with one intermittent SSR panic, see below).

### Admin page holistic de-jank — ACTIVE NEXT STEP
- **Severity:** High (user-facing; user called current admin "janky as hell")
- **Detail:** Current main (`bb078b2`) carries a LIVE regression: the invite-codes `.data-table--invite { table-layout: fixed }` hand-tuned widths **overprint columns** on used rows desktop (status badge over redeemer; code over distributor). Mobile phase-stepper still clips steps 1/5. Root cause (atoms): **a 5-column data table does not fit the 65ch `.prose-page` container** — 8 rounds of width-tuning failed because the FORM is wrong, not the dimensions.
- **Action:** Holistic harsh admin design audit (use `guidance/ui-review-prompt.md`, all 14 admin states 03/04/08/09/10/12/15/18/19/20/21/22/25/27 × desktop+mobile) → form-first fix: invite-codes table → cards / fewer columns / break the 65ch cap for admin tables (REPLACE the table-layout:fixed approach); mobile phase-stepper → compact "step N/5" indicator (not a horizontal 5-step strip). Then holistic re-verify, NOT per-fix.

### Leptos SSR reactive-disposal panic (intermittent)
- **Severity:** Medium (intermittent; likely source of the recurring tower_http 500s in every e2e run)
- **Detail:** `reactive_graph: Tried to access a reactive value that has already been disposed`. `completeOnboarding`'s SSR request leaves a reactive scope a tokio worker disposes after the response; a concurrent request (`validate_invite_code`) hits the dead worker → "Failed to fetch" → fatal once (killed visual-audit test 84, 1 fatal in ~9 reshoots). NOT caused by the CSS/visual work (those commits are CSS + one class-string). See `recon/2026-06-24/e2e-failure-investigation.md`.
- **Action:** Leptos SSR lifecycle investigation of the completeOnboarding reactive-scope disposal.

### Empty-address-enrollment validation gap
- **Severity:** Low. **Detail:** `enroll_in_season(use_existing_address=false, ...)` with both city + np_number empty silently enrolls with an empty address. Pre-existing; exposed by the F2 explicit-flag refactor. **Action:** add server-side validation (reject empty address when not using existing).

### page-09 terminal-season visual-audit coverage gap
- **Severity:** Low. **Detail:** The visual-audit can't reach the terminal-season create-form state in the serial flow without breaking downstream tests; capture 09 was renamed to `admin-no-season-create-form-available`. A distinct post-complete/cancel create-form capture is missing.

### Finalize pending (after admin de-jank)
- 3 consecutive green `just e2e` on the final admin code; fix doc-drift at `guidance/frontend-protocol.md:185` (stale `.stat-card` reference after dead-class removal, via OPUS + full chain); run session-close. **~44 commits unpushed on main — push HELD** (unattended; awaiting user decision).

## Added 2026-06-25 (sectioned review + pipeline)
- **Admin de-jank NOT integrated:** worktree `agent-af224604530d0e0d3` @ `0dfc5ff` (invite-card form, button/count/badge/cancelled-stepper fixes, compact mobile stepper, even card heights). Awaiting capture → spec(opus) → quality(sonnet) → integrate per the Visual Development Pipeline (conventions.md). Push HELD.
- **Automate mechanical visual checks as Playwright assertions** (clip/overflow/overprint/element-past-viewport at 375px + desktop) — deterministic, free per run; reserves agent eyes for aesthetic judgment. Not built. The invite-table overprint + stepper clip should have been assertion-caught.
- **Section-screenshot crops unreliable for clip detection** (DPR crop-window offset fabricates/hides clips). Judge clips on full-page shots only.
- **Whole-app re-verify owed** post-integration — shared CSS changed (toast/headings/stepper); confirm auth/onboarding/home didn't regress.
- **Empty-address-enrollment validation gap** (pre-existing) still open.
- **cycle-viz at 11–15 nodes unverifiable** — seed has 3 nodes; full-cohort label collision uncaptured.

## Updated 2026-06-25 (campaign close)

RESOLVED: admin de-jank integrated + pushed + **CI green** (`405b955`); whole-app re-verify IMMACULATE; 3 greens; doc-drift `.stat-card` fixed; sqlx offline cache regenerated.

Still open:
- Automate mechanical visual checks (clip/overflow/overprint/element-past-viewport at 375px+desktop) as Playwright assertions — not built; the invite-table overprint + stepper clip should have been assertion-caught.
- Empty-address-enrollment validation gap (pre-existing).
- cycle-viz label collision at 11–15 nodes unverifiable (seed has 3).
- Leptos SSR reactive-disposal panic (intermittent).
- IP-based rate limiting absent.
- page-09 terminal-season create-form capture gap.
