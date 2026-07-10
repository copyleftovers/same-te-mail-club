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

## Added 2026-07-03 (complete visual review)

### VISUAL DEFECT BACKLOG — `recon/2026-07-03/DEFECT-CATALOG.md` (24 fix-units)
- **Severity (by fix-unit, max-of-members):** 2 BLOCKER + 15 MAJOR + 6 MINOR + 1 NIT. ~150 raw findings → 34 unique. Full detail in the catalog (gitignored recon — relocate if it must persist past recon cleanup).
- **7 systemic roots (fix first — one systemic fix clears many of the ~55 MAJOR page findings):**
  - **S1 Badge token system incoherent (BLOCKER):** `revoked`=red = destructive-button red; open-code blue = season-`confirmed` blue; 3 terminal states (used/revoked/deactivated) across 2 colors, no rule. THE user's core "conflicting functions share a treatment."
  - **S2 "ДЕАКТИВОВАНО" in actions column (BLOCKER):** badge-as-disabled-button; active row = red button vs dead row = gray pill/bare text. The literal "4 pills for 2 functions." Includes an unstyled-reachable-state.
  - **S3 Primary CTAs styled `secondary` (BLOCKER-grade):** "Далі" (advance) / "Застосувати" (apply) recessive outlines while destructive "Скасувати" is loudest → inverted hierarchy.
  - **S4 Auth no page identity (MAJOR):** zero `<h1>` on any auth step; hero logo at `h-20` → wordmark ~17px pink-on-orange, invisible → two indistinct marks per screen.
  - **S5 `<h3>`/heading drift (MAJOR):** bare `<h3>` renders 3 ways (browser-default / `.sms-trigger h3` / card name); lone `.overline-label` `<h2>` inverts hierarchy.
  - **S6 `.deadline` contract broken (MAJOR):** flat `<p class="deadline">` at all 3 sites; the flex label/value structure + `data-urgency` variants entirely dead (see dead-CSS below).
  - **S7 Rhythm rule fails at non-`<section>` boundaries (MAJOR):** `.prose-page > section + section` breaks where `<div>`/`<article>` interrupt the sibling chain; also two receipt-form `.btn` render flush (no `.btn + .btn` gap).
- S8–S14 (CTA-width auth-vs-home, frozen-toast, sm-button collapse+touch-risk, admin no-sectioning/terminal-framing, mobile table no-degrade, subsection-label hierarchy, stepper terminal states) + 10 LOCAL units in the catalog.
- **Recommendation:** systemic-first via implementer→spec→quality loop.

### Coverage gaps surfaced by the review
- **Dark mode ENTIRELY uncaptured (biggest):** visual-audit spec shoots light-only, but `prefers-color-scheme: dark` reassigns semantic tokens. B5 flags a likely `.alert` AA-fail in dark (color-error text on near-black, est. ~2.5–3:1). Add dark-mode captures to the visual-audit spec.
- cycle-viz only ever seen at ~3 nodes, never a full 11–15 cohort (label collision unverifiable — recurring gap).
- 8 VERIFY-NEEDED items (contrast ratios, touch-target px, font-weight) — screenshots can't confirm. → the standing "automate mechanical visual checks as Playwright assertions" priority would close these deterministically.
- page-09 terminal create-form; empty-state paths.

### Manifesto subagent-oath hook not firing (process/binding integrity)
- **Severity:** Medium. **Detail:** spawned `general-bound` agents report "0 constitution elements bound" — the SubagentStart hook is not injecting `.manifestos.yaml` oaths into spawned agents. Output was unaffected this session because the dispatch/contract carried the constraints, but the oath-propagation mechanism (`.manifestos.yaml` → subagents) is broken. **Action:** investigate SubagentStart hook wiring — does it match the `general-bound` agent type / this session's spawn path?

## Added 2026-07-04 (visual-fix campaign)

### Non-blocking residuals (post-fix, from rendered re-verify)
- **sms-trigger dark border** uses raw brand-gray-20%α, not `--color-border` → weak divider on dark. Trivial: route through `--color-border`; needs a dark re-capture to re-verify.
- **Participant-table long-name row-height asymmetry** — a very long double-barrel Ukrainian name wraps to 2 lines in the ІМ'Я column → that row ~2x taller. The wrap fix works (no overflow/overprint/clip); what remains is height disparity — a multi-column-table-vs-real-name FORM trade-off. Needs a user decision: truncate+title / uniform min-height / accept.
- Pre-existing MINOR: OTP error text centers on mobile.

### Lower-priority tail (not started; all in recon/2026-07-04/DEFECT-CATALOG-v2.md)
- Admin error-corpus i18n (~15 strings across season/assignments/invite_codes/sms; leave infra/500-class English as in login).
- S8 (CTA-width auth-vs-home), S9 (frozen-toast), S10 (sm-button collapse/touch), S11 (admin no-sectioning/terminal-framing), S13 (subsection labels), S14 (stepper terminal states); 12 LOCAL units; MINOR/NIT.
- cohort-15 density capture still deferred (invasive to serial narrative; long-content covers per-item overflow); cycle-viz label collision at scale uncaptured.

### Push HELD
- main @ 7795997, ~10 fix commits unpushed. Awaiting user decision. After push: verify CI (offline-clippy already effectively covered — no query change, but re-confirm).

## Updated 2026-07-04 (fix-everything phase — Wave 1 done, Wave 2 queued)

RESOLVED/verified since the campaign-milestone entry: RV-1, S6-stripe, S10, S12, L12 verified already-adequate; S14 fixed in Wave 1; RV-2 accepted as a structural non-issue (real Ukrainian names in a multi-col table — wrap works, no clip). Wave 1 (CSS) = coherent status-color system + .admin-section + L5 + S14, branch fix/css-wave2 @ ae39300 (spec PASS, quality pending, NOT integrated).

STILL OPEN — Wave 2 (queued, NOT started; session STOPPED here per user):
- admin/page.rs: L1 (theme input aria-invalid — a11y BLOCKER), S11-markup (wrap in .admin-section), S13 (overline labels + filter <label>), L7 (pre-launch cancel + count), L8 (SMS-report→badge/co-locate/zero-count), L10 (cancel-initiate→secondary), L11 (verify invite-card heights).
- admin i18n (~15 strings): season.rs/invite_codes.rs/assignments.rs/participants.rs/sms.rs + uk.json → reasonable Ukrainian via td_string! + strip_server_error_prefix.
- home.rs: S8 (CTA w-full), L4 (address→labeled block), L6 (no-season .empty-state).
- onboarding.rs: L2 (per-field aria-describedby/ids), L3 (centering collapses form → scope centering, w-full form).
- login.rs: L9 (OTP max-width ~12ch), S7-mt3 (remove mt-3 at 795/1003).
- toast.rs: S9 WHOLE (keyframe + auto-dismiss ~4s + suppress on confirmation-destination states) — the CSS-only half was pulled from Wave 1.
- tiny CSS: RV-3 (.field-error text-align:start), V4 (.field-input ::placeholder color).
Then: re-capture + rendered re-verify.
Full detail: recon/2026-07-04/REMAINING.md + DEFECT-CATALOG-v2.md.

## Updated 2026-07-04 (Wave 2 complete — session 7c7c3839)

### RESOLVED this session (from the 2026-07-04 Wave-2-queued list)
All 7 Wave-2 units integrated (admin markup, admin i18n, home, onboarding, login, toast S9, CSS+docs). Two additional rendered-re-verify-caught fixes (login OTP error-text wrap, invite-card mobile stack). 1 E2E capture (onboarding per-field error). Non-blocking residuals and lower-priority tail from REMAINING.md remain open.

### Onboarding participant validation errors are English (i18n gap, pre-existing)
- **Severity:** Medium
- **Detail:** `onboarding.rs` server-fn returns "city is required" / "branch number must be positive" in English. These are participant-visible. COUPLED: `rejected_field_from_error` classifier (D unit) depends on the English substrings; localizing requires re-routing by error code/enum instead of substring match.
- **Action:** Re-route by a typed error variant; then localize the strings.

### field-error in dark mode — coverage gap (RV-3 partial)
- **Severity:** Low
- **Detail:** rv-dark found no `.field-error` in any of the 28×2 dark screenshots (the serial capture flow never renders an error state). `text-align:start` fix was verified in light; dark unverifiable without a targeted dark error-state capture.
- **Action:** Add a dark-mode error-state capture to visual-audit.spec.ts (or extend the existing error-state captures with dark-mode emulation).

### No-season empty-state capture gap (pre-existing)
- **Severity:** Low
- **Detail:** The closest screenshot is `11-home-enrollment-not-open` (season exists but unlaunched). The true "no season at all" home state isn't captured distinctly in visual-audit. User-deferred.

### Mechanical visual assertions still unbuilt (pre-existing, recurring)
- **Severity:** Medium
- **Detail:** No clip/overflow/overprint/element-past-viewport/`scrollWidth<=innerWidth` assertions anywhere in visual-audit.spec.ts. Agent eyes remain the only geometric gate. Confirmed again this wave: the invite-card mobile balloon and login error-text wrap were caught only by rendered re-verify, not code/spec review.
- **Action:** Add Playwright assertions for geometric invariants.

### 7 pre-existing orphan uk.json keys (spec-w2-i18n finding)
- **Severity:** Low
- **Detail:** spec-w2-i18n found 7 orphan key candidates predating this wave's diff. Separate from `dashboard_enrolled_label` (removed this wave). Harmless; no user-visible impact.
- **Action:** Cleanup pass; grep + remove.

### `waitForTimeout(LAYOUT_REFLOW_MS)` in visual-audit `captureElementState`
- **Severity:** Low
- **Detail:** Pre-existing. Technically violates the `waitForTimeout` ban but defensible as a paint-timing settle in a capture harness (not a test assertion wait). Document the exception explicitly if/when the file is refactored.

### sms-trigger dark border (non-blocking residual)
- **Severity:** Low
- **Detail:** `.sms-trigger` uses raw brand-gray 20% alpha for the divider, not `--color-border`. Weak divider in dark mode. Trivial token route.
- **Action:** Route through `--color-border`; dark re-capture to verify.

### Participant-table long-name row-height asymmetry (RV-2, accepted)
- **Severity:** Low (form trade-off, not a defect)
- **Detail:** A very long double-barrel Ukrainian name (e.g. "Олександра-Вікторія Кравченко-Мельниченко") wraps to 2 lines in the ІМ'Я column, making that row ~2× taller than adjacent rows. Wrap works; no overflow/overprint/clip. Multi-column-table vs real-name FORM trade-off.
- **Action:** User decision: truncate + title-attr / uniform min-height / accept.

### Carry-forward (still open from prior sessions)
- Leptos SSR reactive-disposal panic (intermittent tower_http 500s, no test failures).
- IP-based rate limiting absent.
- cycle-viz label collision at 11–15 nodes unverifiable (seed has 3).
- page-09 terminal-season create-form capture gap.

## Resolved 2026-07-05 (follow-up)
- **Onboarding participant errors English** — RESOLVED (c487247+f3291e5): localized via language-independent field routing (server field-key + td_string! message; client routes by key). Ukrainian pixel-verified, CI green.
- **7 pre-existing orphan uk.json keys** — RESOLVED: removed 8 zero-ref keys (the 7 + onboarding_error_prefix).

## Added 2026-07-09/10 (visual-intervention session — checkpoint)

### BLOCKER — auth /login SSR abort (T02+T03 unintegrated)
- **Severity:** High (blocks the last redesign thread from landing). **Detail:** the auth worktree (`.claude/worktrees/agent-ad20d8abde56c5c2e` @ bd7c4d9) release binary ABORTS during SSR of `/login` (Abort trap 6, empty response). Root: `set_interval_with_handle` (client-only) reached the SSR path via `start_cooldown`. Fix `#[cfg(not(feature="ssr"))]` gate (bd7c4d9) applied + spec/quality r3 PASS + SSR clippy clean, BUT recapture STILL aborts. Unresolved hypotheses: (A) harness served STALE pre-fix binary; (B) cfg-gate ineffective — release SERVER build may not enable `feature="ssr"`; (C) a different panic (poss. Leptos SSR reactive-disposal). Debug agents hit session limits. Artifacts: `recon/2026-07-09/reviews/auth-login-ssr-rootcause.md` + `scratchpad/auth-capture-r2.log`. **Action:** resume root-cause (fresh agent seeded from artifact) → fix → recapture-proves-render → rendered-verify → rebase onto main → integrate.

### isolated-capture harness: exit-code gap (T-INFRA-FIX, task #10)
- **Severity:** Medium. **Detail:** `scripts/isolated-capture.sh` exits 0 even when ALL playwright captures fail (empty screenshots). Must propagate `npx playwright test` non-zero exit and/or assert a min screenshot count. Caught only by rendered-verify agents noticing empty dirs.

### main UNPUSHED (6 threads integrated)
- **Severity:** Medium (process). main @ 17e9891 carries harness + T01 + admin(T04/T05) + participant(T06/T07), no CI run, not pushed. Push pending user decision + (after auth lands) whole-app final verify. When pushing after query changes: re-confirm CI-way `SQLX_OFFLINE=true cargo clippy --no-default-features --features ssr` (admin U4 was locale-only; likely no query change, but verify).

### Proposed Ukrainian copy awaiting user veto
- **Severity:** Low. Participant + resend copy shipped as PROPOSALS in uk.json: home_enroll_invitation/expectation/deadline, home_enrolled_milestone, login_resend_code_button/cooldown. Surface for user veto/override.

### Orphan worktree css-systemic-fixes still present (relates to #4)
- Prior-session worktree `.claude/worktrees/css-systemic-fixes` @ cb4f3db + branch `worktree-css-systemic-fixes` (unmerged S1/S4/S5/S6/S7) + branch `capture/dark-and-long-content` (unmerged, dir gone). Decide merge/discard.

## Added 2026-07-10 (auth + T08 + #10 close)

- **cycle-viz long-name top-overflow on mobile** — Low. Pre-existing before T08 (unchanged by viewBox fix): a very long participant name at the ring-top can exceed the SVG top edge at 375px. Revisit with the 11-15-node cohort capture (still uncaptured, carry-forward).
- **`5.0` arrow-clearance literal in render_cycle_ring** — Nit. Quality r2 deferred; name it on next touch of admin/page.rs geometry.
- **Orphan Postgres DB `samete_ssr_debug2`** — Nit. Leftover sibling DB from a prior session's debug run (reported by #10 implementer). `psql -c 'DROP DATABASE samete_ssr_debug2;'` at convenience.
- **RESOLVED: #10 harness exit-0 gap** — isolated-capture.sh now propagates playwright exit + screenshot floor (main @ 47226e8).
- **RESOLVED: auth /login SSR abort (T02+T03 blocker)** — three-layer root cause; integrated main @ 5da012b. Rendered CLEAN.
- **RESOLVED: T08 cycle-viz desktop scale-down** — viewBox 600×560 + aspect-ratio match (main @ 443efdd).
- **Resend + participant UK copy proposals** — user explicitly deferred review 2026-07-10; keys live as proposed (login_resend_code_button/cooldown, home_enroll_*, home_enrolled_milestone).
- **RESOLVED: orphan branches (#4)** — 85 stale branches pruned with patch-identity evidence; repo single-branch.
