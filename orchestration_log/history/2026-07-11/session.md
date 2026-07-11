# Session 2026-07-11 — T-FIX: app-defect fix campaign (thread 2 of the fault-intake session)

Mandate: fix the 7 actionable rendered-verify defects (A1-A6, A9 from history/2026-07-10/app-defect-catalog.md), then verify component-system propagation of all session UI changes.

## Checkpoint — close (T-FIX CLOSED)

### Outcome
main @ 4a9792f PUSHED; CI green ×2 (29148792346 @ 2e001a5, 29149491267 @ 4a9792f). 13 commits over 01878df. All 7 defects CLEARED in rendered pixels; component-propagation audit PASS after drift fixes.

### Threads (5 parallel worktrees → sequential rebase-integration)
- A1 cycle-viz label collisions: quadrant-aware text-anchor + radial offset + two-line tspan + <title> tooltip + viewBox 900×750. Implementer ran its OWN cohort-capture pixel loop (2 iterations). Spec reviewed WITH independent pixel verification. n=15 ceiling: <title> fallback.
- A2 404: page-frame/prose-page/empty-state idiom + 2 uk.json keys.
- A3 toast (BLOCKER): root cause sticky-in-main pinning mid-viewport on scroll → in-flow block between Header and <main>; all 4 historical overlay collisions geometrically impossible; z-index dropped (flow needs none).
- A4 create-season error borders: FIELD_DISCRIMINANT_SEPARATOR idiom ported from onboarding; exactly one field flagged; infra errors banner-only.
- A5/A6/A9 dark polish: .info-link accent→--color-text; --color-panel-dark capped at 0.27 (0.28 FAILS AA for error text 4.44:1 — implementer's doc-sync math caught what spec r1 accepted; independent recomputation confirmed 4.59/6.51/14.22 on 0.27); A6 first no-op'd on correct-but-misleading math (4.7:1 ratio ≠ perceptible for a 13px hairline glyph) → pixel-first re-fix: dark uses --color-text (~16:1).

### Component-propagation audit (user-mandated gate, post-green)
Verdict: PROPAGATED after fixing 1 MAJOR + 2 MINOR: testid-as-CSS-selector at TWO sites (one pre-existing) → semantic .inactive-marker class; stale frontend-protocol z-index map (toast entry removed); .info-link documented in design-system §Components. NOTEs deferred: FIELD_DISCRIMINANT_SEPARATOR defined in 2 modules + idiom undocumented (promote on 3rd usage).

### Lessons
- Computed contrast ratio ≠ perceived visibility for hairline glyphs (em-dash at 13px): pixel evidence overrules AA arithmetic for sub-glyph strokes.
- Diff-baseline illusion: after mid-wave integration, `diff main..HEAD` in stale-base worktrees shows OTHER units' changes as reverts — two reviewers false-flagged scope creep. ALWAYS diff against merge-base; bake into every reviewer brief once integration starts.
- Doc value-sync is a defect detector: forcing recomputation of documented ratios (0.28 sync) exposed a real AA failure both implementer and spec r1 had passed.
- Reviewer-vs-implementer contradiction (A3 mount): arbitrate with one primary-source git query, not re-reads of either report.
- Styling-via-testid = inverse coupling of the testid ban in tests; semantic classes only. (Now enforced: grep 'data-testid' style/ = 0.)

### State
Repo single-worktree, clean, pushed, CI green. Tasks #12 #13 complete. No crons live (insurance one-shots 17:17/17:47 auto-expire). Standing open: A10 design question (unlaunched season invisible — user veto/decide), cohort-seed.sql LIKE/UUID idempotency-guard nit, F4/F5 NOTEs, copy-review deferral (uk.json proposals), pre-existing intermittent SSR 500.
