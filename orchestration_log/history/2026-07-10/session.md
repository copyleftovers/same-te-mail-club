# Session 2026-07-10/11 — T-CAP: screenshot-capture elevation (fault-intake session, thread 1)

User mandate: audit whether `just capture-isolated` screenshots are fit for review (all states, meaningful); if not, elevate to completeness — fully completed, integrated, verified, no interim reports.

## Checkpoint — 02:02 (post session-limit wake)

### Narrative
1. DISCOVERY (parallel sonnet): capture-inventory (224 files/33 states; fragile shared-SEQ naming; 102 section crops = 45% noise; dark set clean-states-only) + app-state-space (55 stable + 49 transient states, source-cited). Both stalled once (stream watchdog), resumed via SendMessage, completed.
2. DESIGN (opus): `recon/2026-07-10/capture-gap-design.md`. Verdict: NOT fit. 14 CAPTURE-GAPs, 4 dark-error gaps, 1 FLOW-GAP, 1 SEED-GAP, 1 prune. Orchestrator REJECTED designer's SEED-GAP (12-node cohort cycle-viz) exclusion → revised: Unit 7 SQL-seeded cohort pass, feasible (~60-line seed, no OTP walking). 7 units total. All 3 flagged testids already exist in source — ZERO Rust changes. VISUAL_SPEC one-line harness override approved.
3. IMPLEMENT attempt 1 (single implementer, 7 units): died instantly — "Prompt is too long" at 9 tool uses (oversized read suspected). Worktree pruned (nothing committed). Lesson: read-hygiene rules now MANDATORY in implementer briefs (chunked reads ≤400 lines; never read package-lock/node_modules/target/screenshots/logs; pipe noisy cmds to file + tail).
4. IMPLEMENT attempt 2 (split, parallel, disjoint write-sets):
   - A (units 1–6, visual-audit.spec.ts): 3ac07bf + 2c20e5c. Spec r1 FAIL (L7 not distinct; L11 used-code error captured BEFORE code consumed — server would accept). Fix f6e6606. Spec r2 PASS.
   - B (unit 7, cohort pass): 182e838. Spec r1 FAIL (cohort spec unguarded → default `just e2e` discovery would seed a season+12 users into shared samete DB). Fix 816a070 (file-scope test.skip(COHORT_CAPTURE!=="1") blocks beforeAll; assertSiblingDatabaseUrl throws on unset/samete; harness case-block exports COHORT_CAPTURE=1 only for cohort VISUAL_SPEC). Spec r2 PASS. Quality r1 Ready-Yes (3 Minors). Fold Minor-1: f0f92cb (bare ON CONFLICT DO NOTHING ×5 inserts, header-documented deviation from test_admin.sql targeted form). Minor-2 (viewport/reflow consts duplicated from visual-audit.spec.ts) DEFERRED (cross-file). Minor-3 skipped (matches precedent).
5. SESSION LIMIT (reset 02:00 Kiev) killed: units-1-6 quality reviewer adb21e8b0efa811f8 (died at 7 tool uses) + unit-7 fold-confirm continuation a0caa19ea76555b1f. Both resumable (rate-limit class).

### Working state (RESUME HERE)
- main @ d06b4ba, clean, untouched. Worktrees intact + clean: agent-a985803aba638e3fc (units 1–6 @ f6e6606, spec PASS, quality INTERRUPTED), agent-a8472f78b17f4b437 (unit 7 @ f0f92cb, spec PASS, quality Yes, fold-confirm INTERRUPTED).
- NEXT: (1) resume quality review units 1–6 (fresh or SendMessage adb21e8b0efa811f8) + fold-confirm unit 7 (SendMessage a0caa19ea76555b1f); (2) on both green: CAPTURE VERIFICATION — background bash from worktree A merged view… NO: integrate order = A then B onto main (disjoint files, rebase-clean), then run BOTH passes from main via isolated harness: `bash scripts/isolated-capture.sh tcap visual` + `VISUAL_SPEC=tests/visual-audit-cohort.spec.ts bash scripts/isolated-capture.sh tcapcohort visual`; gate on screenshot count + INDEX.md present; (3) rendered-verify agents (one dir per agent, ≤30 imgs each) judge fitness incl. 12-node cycle-viz label collision; (4) task #12 complete. Alternative: capture BEFORE integrate from worktree A with B's two files cherry-picked — rejected, integration-on-green is cleaner (both units independently reviewed).
- Verdict files: recon/2026-07-10/reviews/spec-*-a8472f78b17f4b437-{120000 FAIL,001500 PASS}.md, spec-*-a985803aba638e3fc-{120000 FAIL,063000 PASS}.md, quality-*-a8472f78b17f4b437-210342.md (Yes; fold-confirm pending append).
- Crons: stale backstop 64c5a30a to delete; wake+5h insurance pair to set (07:02 / 07:32 one-shots).

### Decisions
| Decision | Rationale |
|---|---|
| Reject designer's cohort-pass exclusion | Owner's core need = states unreachable manually; sibling DB permits direct SQL seed |
| Split implement into 2 disjoint-write-set agents | Attempt-1 context death; parallel + smaller input load |
| Fold only Minor-1 on unit 7 | Cheap + precedent (test_admin.sql); Minor-2 cross-file (deferred), Minor-3 precedent-consistent |
| Integrate A then B, capture from main | Both independently reviewed; disjoint files rebase clean |

### Failures
| Failure | Root cause | Correction |
|---|---|---|
| Implementer 1 "Prompt is too long" @ 9 tool uses | Oversized single read (suspected package-lock/large file) | Read-hygiene rules in every implementer brief |
| L11 capture precondition (spec r1) | Capture before code consumption — server accepts | Restructured test placed after genuine consumption |
| Cohort spec unguarded (spec r1) | Playwright default discovery runs all tests/*.spec.ts | COHORT_CAPTURE opt-in + samete-refusal guard + harness export |

## Checkpoint — 04:50 (T-CAP CLOSED)

### Outcome
Thread T-CAP complete: main @ d6a17f6 PUSHED, CI run 29132991848 SUCCESS. 18 commits over d06b4ba, all through spec→quality→pixels gates. Capture suite verdict elevated from UNFIT (fragile SEQ names, 45% crop noise, no dark-error coverage, 14 unshot states, no cohort scale) to FIT (delta-verified).

### Shipped
- visual-audit.spec.ts rewrite: stable `{area}-{state-slug}[__variant]` names (order-independent, diffable), 4-scheme matrix (light/dark × desktop/mobile) incl. error/focus variants, atomic recordScreenshot→MANIFEST (one page.screenshot site), INDEX.md manifest (afterAll), stale-png cleanup, section crops pruned 102→3, 11 gap-fill captures, Pass B (revoked/deactivated mutations) + Pass C (existing-address enroll, pre-submit not-received form H7b).
- visual-audit-cohort.spec.ts (NEW): SQL-seeded 12-node cycle-viz pass (60-line seed, no OTP walking; double-barrel Ukrainian names) — COHORT_CAPTURE=1 gated (file-scope test.skip blocks beforeAll under default discovery) + assertSiblingDatabaseUrl refuses samete + harness auto-exports flag for cohort VISUAL_SPEC. Idempotent seed (bare ON CONFLICT DO NOTHING, documented) + idempotent INDEX row merge.
- isolated-capture.sh: VISUAL_SPEC env override (default path untouched).
- Final artifact: 174 pngs == 174 INDEX rows (two-direction clean), 43+1 tests green ×2 invocations; full-suite CI-way (mode=full, isolated) 117 passed / 2 by-design skips.

### Unreachable-states doctrine (5 removals, all source-proven)
L2 phone-error + L15 name-error (anti-enumeration: invalid input → Ok(AccountExists)/redirect, never DOM error), O2 city-empty (HTML5 required blocks POST), A40 same-participant swap (double-update no-op → Ok), L7 resend-active (cooldown seeds on step activation → only cooldown state exists; capture deduped vs L5). LESSON: the opus gap-design inferred states from DOM elements without tracing server semantics — 5/16 designed gap-fills were phantom. Rendered-verify caught what 4 review rounds missed.

### App defects surfaced (capture suite doing its job) → orchestration_log/history/2026-07-10/app-defect-catalog.md (tracked copy)
7 actionable: A1 cycle-viz 12-node label collisions (3 zones, all modes/viewports — the weeks-deferred unverifiable item, NOW PROVEN), A2 404 unstyled, A3 toast splits/obscures admin content (BLOCKER-rated), A4 error borders on ALL create-season fields incl. valid (dark), A5 phone numbers accent-orange, A6 deactivated-row em-dash contrast (dark), A9 alert fill imperceptible. +A10 design question (unlaunched season invisible to participants — home.rs:276 launched_at IS NOT NULL). A7/A8 resolved NOT-A-DEFECT with citations. Dark token system verified CLEAN across all 42 documented dark-desktop files.

### Process lessons
- "Prompt is too long" at ~9 tool uses = single oversized read, not accumulation; read-hygiene rules (chunked reads, no package-lock/target/screenshots, pipe+tail) now standard in implementer briefs.
- Fix-run-fix LOOP agent (owns capture iterations in its worktree) beats one-failure-per-orchestrator-cycle — 3 defects cleared in one dispatch vs 2 cycles for the first 2.
- Reviewer window ceiling: spec reviewer retired at ~269k, quality at ~201k (sonnet 250k) — fresh-launch successors seeded from verdict files; NEVER recycle image-heavy or near-ceiling agents.
- `&&`-chained background bash with trailing `; echo` masked a failing exit — chain with && only, no trailing commands.
- Stream-watchdog stalls (600s) at the write step: resume via SendMessage with write-only delta — both discovery agents recovered losslessly.
- VISUAL_SPEC=<any spec> + mode=full turns the isolated harness into a full-suite CI-way validator (117 tests, zero :3000/samete exposure).

### State
Repo: single worktree, main @ d6a17f6 pushed, CI green. Crons: all deleted except 07:02/07:32 one-shot session-limit insurance (auto-expire). Task #12 → completed. Deferred-items delta: cycle-viz cohort-capture gap RESOLVED (now captured + defect proven); new: app-defect catalog (7+1 entries) awaiting user fault-intake.

## Close pointer

Session continued into 2026-07-11 (same session-id); close record + quantitative summary live in ../2026-07-11/session.md.
