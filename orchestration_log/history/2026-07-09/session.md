**Orchestrator:** Claude Fable 5 (upgraded in-session from Opus)
**Session ID:** 96328b0b-ac13-41e9-a973-afc4ea2f3c2e
**Duration:** ~42.7h wall (2026-07-09T17:02Z → 2026-07-11T11:46Z, spans 3 calendar days + 2 session-limit resets)
**Cost:** see local `cost.md` (gitignored; per-session) — not yet captured: `/cost` is interactive-only; run it and paste verbatim into `orchestration_log/history/2026-07-09/cost.md`
**Code changes:** 1176 insertions, 356 deletions across 17 files (35 commits d06b4ba..80d1c70)
**Outcome:** capture suite elevated to complete reviewable coverage (T-CAP) + all 7 rendered-verify app defects fixed (T-FIX) + component-propagation audit clean + UK copy finalized + A10 closed; 4 CI-green pushes.

# Session 2026-07-09 — Visual Design-Intervention Campaign (participant + auth + admin)

User walked the app screen-by-screen reporting visual/UX faults; orchestrator ran each as a parallel fault-THREAD (one fault = one thread, spanning many worker agents, closing only on solved-with-no-regressions). Design-intervention scope (not polish), mobile-first, Ukrainian.

## Checkpoint — session-limit park (approx 2026-07-10 ~00:xx)

### Narrative (thread lifecycle: DISCOVERY→DESIGN→PLAN→IMPLEMENT→SPEC→QUALITY→VERIFY(pixels)→INTEGRATE→CLOSED)

- **T01 — participant shell** (header/vertical-rhythm/typography). CLOSED, on main. New `.page-frame` viewport-height layout primitive; short/terminal states center via `.page-frame > .prose-page > .empty-state { margin:auto }`; header rebuild (bigger mark, logout=quiet pill, admin-nav=text link); `--size-logo-height-header` token; `.empty-state .empty-state-headline` specificity fix (--text-section beats `.prose-page h1`). Header shared across participant/admin/login/onboarding — fix kept participant-home-only (min-height calc, not touching shared `<main>`). Pixel-verify FAILED TWICE before passing (see Failures).
- **T04 — admin Season Management** (folded faults #5 typography-drift, #6 pre-launch-counter-IA, #7 SMS-report-bar+button-hierarchy). CLOSED, on main. Typography consolidated to canonical roles; status→aligned `.badge`; counter unification; per-phase IA (3-way gate on `season.launched`, page.rs:417/state.rs:180 — pre-launch hides always-0 enrolled/confirmed + shows pool count as metric row); `.sms-report-result` margin rhythm; 4 SMS buttons→secondary, "Далі" sole primary. created-vs-launched distinction KEPT (draft/live boundary).
- **T05 — BUG active-user count → 0 after SMS send**. CLOSED (folded into T04 U4). Was a MISLABEL not wrong-value: `season_open_target_count` = active users NOT yet notified (correctly 0 after all sent); relabeled `sms_count_active_users`→"N ще не отримали", stripped decorative arrows.
- **T06 — participant EnrollmentOpen** (fake-field/typography/intent). CLOSED, on main. Fake input → honest `.info-list` labeled block (address is display-only; enroll_in_season ignores it when use_existing_address=true); deadline elevated to a time-boxed decision-constraint sentence; intent reorder. Proposed UK copy keys (home_enroll_invitation/expectation/deadline) — user-veto pending.
- **T07 — participant Enrolled "Ти в списку"** (temporal-expectation/no-action-conflict). CLOSED, on main. Enrolled is Phase::Enrollment; confirm_ready needs Phase::preparation → NO action available now. Structurally zero CTA (Gate C3: no button/ActionForm/.deadline); forward-milestone copy (home_enrolled_milestone). Centered `.empty-state`. Removed 2 orphan i18n keys.
- **T-INFRA — parallel isolated capture harness**. CLOSED, on main. `scripts/isolated-capture.sh` + one-line `playwright.config.ts` CAPTURE_BASE_URL fallback + `capture-isolated` justfile recipe. Mechanism: bypass `cargo leptos end-to-end` (it hardcodes :3000 + chains stomping `_kill-stale`+`db-reset`); build binary + run directly with LEPTOS_SITE_ADDR=free-port + DATABASE_URL=sibling `samete_<suffix>` DB on the live Docker Postgres; playwright via CAPTURE_BASE_URL; trap teardown (kill own PID, drop own DB); hard `!=3000`/`!=samete` aborts. Validated: parallel captures with distinct sibling DBs, `samete` never touched.
- **T02+T03 — auth/login screen + OTP resend**. **BLOCKED, NOT integrated** (see Working State).

### Decisions
| Decision | Context | Rationale |
|----------|---------|-----------|
| Thread = fault; agents = workers | User corrected terminology | Thread closes only on solved-no-regressions |
| Harness task-board → harness Task tools | User scratched agent-managed board | Simpler, communicationally clear, visible shared list |
| general-bound for multi-tool general dispatches; specialized agents own+recycle | User directive | Constitution binding + context-holding continuity |
| Build isolation harness (own port + sibling DB) | :3000/db-reset stomped user's live dev server | The true fix for parallelizable capture, not deferral |
| Foundation-first serialization | T02/T04/T06 all consume T01's shared CSS language | Only the shared CSS foundation is irreducibly serial; all thinking parallelized; per-screen implements branch from T01-integrated main |
| Integrate sequentially, rebase each group on prior main | Shared tailwind.css/uk.json across groups | Worktree-native merge; git auto-resolved distinct keys/regions |
| created-vs-launched KEPT | User asked "useful distinction at all?" | Draft/live boundary is real; fix is what-to-show-per-phase |

### Failures (root causes preserved)
| Failure | Root cause | Correction |
|---------|-----------|------------|
| I ran `just e2e-release` for T01 capture | Pipeline `_kill-stale`(:3000)+`db-reset`(samete) killed user's live dev server + wiped DB | Built isolation harness; standing constraint: never e2e/db-reset/:3000 while user works |
| T01 pixel-verify FAILED x2 (void not fixed) despite spec+quality PASS | (1) `.page-frame` centering inert — `min-height:100%` collapsed on auto-height parent → fixed to `calc(100dvh - --header-height)`; (2) `.page-frame > .empty-state` never matched — `.empty-state` is a GRANDCHILD (`.prose-page` between) → fixed to `.page-frame > .prose-page{flex:1;flex-col}` + `> .prose-page > .empty-state{margin:auto}` | Root-cause debugging on ACTUAL rendered DOM (2 source-reasoning fixes failed first) |
| First root-cause read the WRONG tree (main, not worktree) → "phantom class" | My dispatch gave main-repo paths; T01 unintegrated so .page-frame absent in main | Lesson: worktree-scoped agents get WORKTREE paths |
| Auth capture "exit 0" but 0 screenshots | Harness doesn't propagate playwright non-zero exit (#10) | rendered-verify agent caught empty screenshots; tracked #10 |
| Auth /login SSR ABORT (Abort trap 6) | `set_interval_with_handle` (client-only) ran on SSR path via start_cooldown | cfg-gate applied (bd7c4d9) BUT recapture STILL aborts — UNRESOLVED |

### Working State (RESUME HERE)
- **main @ 17e9891, UNPUSHED.** Integrated: T-INFRA harness, T01 foundation, admin (T04+T05), participant (T06+T07). Clean tree. NO CI run yet, NOT pushed (awaiting user).
- **Auth (T02+T03) BLOCKED, unintegrated.** Worktree `.claude/worktrees/agent-ad20d8abde56c5c2e` @ **bd7c4d9** (branch worktree-agent-ad20d8abde56c5c2e, based on 65dc381 — needs rebase onto 17e9891 at integrate). Code: `.auth-card` + `.btn[data-variant="link"]` + page-frame wrap + back-buttons→link + OTP resend affordance (RwSignal cooldown, `start_cooldown` closure, reuses request_action, no new server fn) + both resend i18n keys. Spec r3 PASS + quality r3 YES + SSR clippy clean + cargo test 50/0.
- **BLOCKER: `/login` release binary aborts on SSR** (Abort trap 6, empty response, all captures skipped, 0 screenshots). cfg-gate `#[cfg(not(feature="ssr"))]` on start_cooldown body+constant (bd7c4d9) did NOT clear it. THREE unresolved hypotheses when parked: (A) harness served a STALE pre-fix binary (didn't rebuild); (B) cfg-gate ineffective — does cargo-leptos release SERVER build actually enable `feature="ssr"`? if not, gate is inverted; (C) a DIFFERENT panic (e.g. known Leptos SSR reactive-disposal). Debug agents (ae4d7c7a → successor a6ff91b) hit SESSION LIMITS mid-investigation. Prior findings in `recon/2026-07-09/reviews/auth-login-ssr-rootcause.md` (5964 bytes, first pass). Failing capture log: `scratchpad/auth-capture-r2.log`.
- **NEXT (on user say-so):** resume auth SSR root-cause (fresh agent, seed from artifact + r2 log) → resolve A/B/C → fix → recapture (prove /login renders) → rendered-verify → rebase onto main → integrate → then whole-app final verify + push decision.
- Remaining worktrees: agent-ad20d8ab (auth, keep — unintegrated), css-systemic-fixes (prior-session orphan, deferred #4).
- All crons cleared. Awaiting user say-so.

## Checkpoint — auth-resume plan (pre-compaction, 2026-07-10)

RESUME EXACTLY HERE. Everything below is self-sufficient with the referenced files; the conversation tail is being compacted.

### Status
- `main @ 17e9891` (UNPUSHED, clean): 6 threads CLOSED — T01 shell, T04 admin season-mgmt, T05 count-relabel, T06 EnrollmentOpen, T07 Enrolled, T-INFRA isolated-capture harness.
- SOLE OPEN THREAD: **auth (T02+T03)**. Worktree `.claude/worktrees/agent-ad20d8abde56c5c2e @ bd7c4d9` (branch worktree-agent-ad20d8abde56c5c2e). Code = `.auth-card` + `.btn[data-variant=link]` + page-frame wrap + OTP resend affordance. spec r2 PASS + quality r2 YES.
- BLOCKER: `/login` aborts during SSR (`Abort trap: 6`, empty response) → isolated capture renders 0 screenshots. The cfg-gate fix (bd7c4d9: `start_cooldown` body + `OTP_RESEND_COOLDOWN_SECS` wrapped `#[cfg(not(feature="ssr"))]`) did NOT clear it.
- 3 live hypotheses: **A** stale binary (harness served pre-fix build) · **B** cfg-gate ineffective (release SERVER build may not enable `feature="ssr"` → gate doesn't exclude the timer) · **C** a different panic (e.g. Leptos SSR reactive-disposal).

### Immediate plan (apply the recycling/context doctrine — DECOMPOSE, do not over-commit one agent)
Launch 3 PARALLEL single-question agents (cheap; the prior over-committed-sonnet debug agent overflowed — do NOT repeat):
1. **Panic (C):** bounded grep/tail `scratchpad/auth-capture-r2.log` for the panic line before `Abort trap: 6`. Still `set_interval` → B. New construct → C. (The r2 capture is ALREADY a fresh isolated repro — harness builds under `set -euo pipefail`, so read its output; don't blindly re-run a multi-min build.)
2. **Binary freshness (A):** `./target/release/samete` mtime vs `git show -s --format=%ci bd7c4d9` + grep r2 log for "Compiling samete" → rules A in/out.
3. **Feature flag (B):** read `Cargo.toml [features]` + `[package.metadata.leptos]` → does the release server binary compile with `feature="ssr"` (so `#[cfg(not(feature="ssr"))]` actually excludes the timer)?

### Then
Assemble 3 answers → targeted fix (recycle the auth implementer ad20d8ab IF still resumable + within window, else fresh-launch seeded) → recapture via isolated harness, **gate on screenshot-count>0 NOT exit code** (harness exit-0 gap = task #10, still open) → rendered-verify auth (fault-02 + fault-03 criteria) → rebase auth worktree onto `main` → ff-integrate the final thread → whole-app final verify (all screens, both viewports+modes) → push decision (user).

### Reference artifacts
- Resend spec: `recon/2026-07-09/fault-03/design-intervention.md`
- First-pass SSR root-cause: `recon/2026-07-09/reviews/auth-login-ssr-rootcause.md`
- The abort repro log: `scratchpad/auth-capture-r2.log`
- Auth design/plan: `recon/2026-07-09/fault-02/{design-intervention,impl-plan}.md`

### Confirmed capability (for the record)
Parallel isolated captures LANDED + exercised: admin + participant captured concurrently on distinct ports + sibling DBs (`samete_admin` & `samete_participant` coexisted, `samete` untouched). Parallelism is ACROSS captures (per worktree); within a capture the visual-audit is serial by design (shared-DB narrative).

## Checkpoint — auth CLOSED + limit-park (2026-07-10 ~18:xx)

### AUTH (T02+T03) CLOSED — INTEGRATED. main @ 5da012b (ff of 13 rebased commits, was 17e9891). UNPUSHED.
Root-cause chain of the /login SSR abort (3 layers, each empirically verified):
1. `StoredValue<Option<IntervalHandle>>` + `on_cleanup` unconditional → web-sys AbortController on SSR disposal → Abort trap 6. Fixed 73275f4 (cfg-gate outside view!).
2. `request_action.dispatch()` in resend on:click ran during SSR pre-warming (abort_signal panic, non-fatal). Fixed dc2d4af, then RESTRUCTURED 9fd53bf: ALL #[cfg] out of view! (cfg inside view! closures breaks macro tokenization), dual signature-identical `resend` bindings (client real / SSR no-op).
3. THE REAL ATTR-LEAK ATOM: bare `>` in unbraced view! attribute expr `disabled=move || cooldown.get() > 0 ...` — macro parses `>` as tag close, ALL following attrs leak as text nodes + step-visibility desyncs. Fixed 410ef0f: brace the expr `disabled={...}`. WHY-comment bdfde27/5da012b.
Also: D1 mobile label containment = .auth-card flex shrink-wrap → `.auth-card > div, .auth-card form {width:100%}` (776d46c); capture-spec hardening 450f0e7+fold (clearCookies, toBeVisible step-gate, restored toBeEnabled hydration wait).
Gates passed: spec r4-r8 PASS, quality r4-r8 YES (0 open findings), capture 35/35 ×2, rendered-verify CLEAN ×2, post-rebase sweeps AUTH/MOBILE/DESKTOP-DARK CLEAN, DESKTOP-LIGHT 1 pre-existing defect (→T08). Debug-server empirics: /login 200 both profiles.

### Lessons (verified this session)
- cfg-attrs ANYWHERE inside view! (incl. closure bodies) = macro misparse. Gate via dual cfg'd bindings OUTSIDE view!, signature-identical, no-op SSR stub.
- Bare `>` comparison in an unbraced view! attr expr eats the tag. Brace attr exprs containing comparisons.
- Release+LTO backtraces misattribute frames (phantom toast.rs). Debug-profile repro before trusting frames. grep-verification of served HTML must be token-spacing-tolerant (leak rendered `on : click`).
- Image-pass agents: ~30 full-page shots ≈ one sonnet window. One dir per agent; never recycle an image-heavy verifier into a second full pass (2 overflows this session).
- Playwright serial capture: session cookies leak across tests → clearCookies before auth-state captures.

### OPEN at park (limit resets 21:00 Europe/Kiev)
- T08 (#11): cycle-viz SVG overflow. Discovery agent af982b39bccf05dae INTERRUPTED mid-run (22 tool uses; artifact recon/2026-07-10/t08/design-intervention.md NOT confirmed written; evidence copy of shot-24 variants to t08/ may be partial). Resume via SendMessage or relaunch; task #11 has full fault spec.
- #10 harness exit propagation: implementer a226e765ee1e107d3 INTERRUPTED early (worktree .claude/worktrees/agent-a226e765ee1e107d3 created, work state unknown). Resume via SendMessage; task #10 + dispatch spec in transcript.
- Auth worktree agent-ad20d8abde56c5c2e: integrated, REMOVAL PENDING (waiting on T08 evidence copy confirmation; screenshots there are the only fresh capture set).
- Push main (user decision). Orphan branches (#4, user decision). Resend UK copy proposals await user veto.

## Checkpoint — T08 + #10 closed (2026-07-10 ~21:xx)

### Narrative
Post-auth-integration, two parallel threads ran full dev chains and INTEGRATED:
- **#10 harness exit propagation** — implementer (worktree) `|| playwright_exit=$?` + screenshot-floor (marker mktemp + `-newer` count; exit 1 on exit-0-with-0-pngs); sabotage-verified BOTH failure paths (fake build binary + no-match grep → exit 1 + teardown ran; `true` playwright → FATAL floor). Sabotage test caught a real pipefail bug in the first cut (find assignment fatal when screenshots/ absent → `|| true` guard). spec PASS, quality r1 YES + Important fold (marker cleanup → teardown trap, r2 YES). main ff → `47226e8`.
- **T08 cycle-viz overflow** — root cause: `render_cycle_ring` (admin/page.rs) hard-coded viewBox 400×400, center(200,200), r=150 → ~50px label margin; Ukrainian labels (~120px half-span) overflow viewBox into layout space → intrinsic width >1280 → whole-page scale-down. Mobile escaped (SVG at 311px keeps absolute overhang under browser threshold). Fix: viewBox 600×560 center(300,260) + `--size-cycle-viz` 400→480px + named geometry constants. Quality Critical caught: CSS `aspect-ratio: 1` vs non-square viewBox → 600/560 + WHY comment; aliases dropped. Pixels CLEAN ×2 (incl. post-aspect-change re-verify). Rebase + ff → main @ `443efdd`.

### Decisions
| Decision | Context | Rationale |
|----------|---------|-----------|
| Integrate auth despite DESKTOP-LIGHT 1 defect | cycle-viz overflow pre-existing; auth CSS .auth-card-scoped | Don't hold a green thread hostage to an unrelated fault; opened T08 as owned thread instead |
| T08 form: viewBox expansion, NOT overflow-x scroll | 3-node overflow was coordinate-space clearance, not true width pressure | Simplest form honoring frontend-protocol (no page-level scroll); 11-15-node behavior stated in design artifact |
| 5.0 arrow-clearance literal left unnamed | Quality r2 reclassified Important→Minor, deferrable | Fold policy: skip reviewer-declined/cosmetic |

### Failures
| Failure | Root cause | Correction |
|---------|-----------|------------|
| Agent spawn failed "Failed to resolve HEAD" | CWD sat inside just-removed auth worktree | cd to project root before spawn; worktree removal invalidates any CWD inside it |
| SendMessage to rendered-verifier failed (no transcript) | Agent reaped between turns | Fresh-launch with self-sufficient brief; reaped ≠ error |
| Both live agents killed by session limit mid-run | Throughput cap (resets 21:00) | Rate-limit class = RESUMABLE: SendMessage continuation restored both losslessly; one-shot cron as fallback resume trigger |

### Working State
main @ `443efdd` (auth 5da012b → #10 47226e8 → T08 443efdd), clean except uncommitted orchestration_log/ docs. UNPUSHED — push is user's call. All worktrees removed; no crons live. Open: #4 orphan branches (user), resend UK copy veto (user), session-close when user says.

## Checkpoint — repo hygiene + close (2026-07-10 ~22:xx)

- **Branch/worktree purge complete:** 85 stale branches deleted (5 zero-ahead + 5 evidence-superseded orphans incl. css-systemic-fixes worktree + 43 patch-equivalent + 32 SUPERSEDED-verified via `git cherry` + main-source signature greps; evidence: recon/2026-07-10/{orphan-branch-triage,stale-branch-prune}.md). Repo now: 1 worktree (main), 1 branch (main). Task #4 CLOSED.
- **Lesson:** rev-list "ahead" counts lie after rebase-integrations; `git cherry` patch-identity is the prune criterion.
- **Resend UK copy review DEFERRED by user** (login_resend_code_button/cooldown + home enroll/enrolled proposals stay shipped-as-proposed).
- Close sequence: push code @ 443efdd → docs commit [skip ci] → CI watch via gh.

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

### App defects surfaced (capture suite doing its job) → orchestration_log/history/2026-07-09/app-defect-catalog.md (tracked copy)
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


# Session 2026-07-11 — T-FIX: app-defect fix campaign (thread 2 of the fault-intake session)

Mandate: fix the 7 actionable rendered-verify defects (A1-A6, A9 from history/2026-07-09/app-defect-catalog.md), then verify component-system propagation of all session UI changes.

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

## Checkpoint — session close (copy + A10 threads)

- **UK copy finalization** (owner delegated judgment): 4 strings changed / 4 kept + 1 sibling voice fix folded (home_season_cancelled_body → ти-form). Reviewed (interpolation vars verified against macro call sites, 187→187 keys), integrated, pushed 80d1c70, CI green 29151064861.
- **A10 resolved by spec trace + user ruling:** NOT-PRESCRIBED (4.1/4.2/4.3 ACs make pre-launch draft admin-internal; adding a state would conflict). User confirmed create/launch stands. Follow-up user probe "is the draft state itself prescribed?" answered YES with citations (4.1/4.2/4.3 + SMS 5.3 timing) — kept.
- Session-limit insurance one-shots 17:17/17:47 deliberately KEPT at close (user directive — they mark the next session window).

## Quantitative Summary

### Agents by model tier

| Model | Agents |
|---|---|
| claude-sonnet-4-6 | 99 |
| claude-opus-4-8 | 23 |
| claude-opus-4-6 | 3 |
| \<synthetic\> | 3 |
| **Total** | **128** |

### Agents by type (top-5)

| agentType | Count |
|---|---|
| general-bound | 63 |
| dev-discipline:implementer | 21 |
| dev-discipline:spec-reviewer | 19 |
| dev-discipline:code-quality-reviewer | 16 |
| general-purpose | 9 |

### Orchestrator tool calls (top-10)

| Tool | Calls |
|---|---|
| Bash | 159 |
| Agent | 129 |
| TaskUpdate | 93 |
| SendMessage | 77 |
| CronCreate | 59 |
| CronDelete | 54 |
| TaskCreate | 13 |
| ToolSearch | 7 |
| Edit | 7 |
| TaskOutput | 5 |

### Token totals (main + subagents combined)

| Model | Input | Output | Cache Read | Cache Creation |
|---|---|---|---|---|
| claude-fable-5 | 6,248,952 | 949,341 | 346,405,372 | 27,309,480 |
| claude-opus-4-8 | 4,309,422 | 2,087,162 | 388,006,235 | 22,386,692 |
| claude-sonnet-4-6 | 14,140 | 736,565 | 456,089,442 | 31,512,565 |
| claude-opus-4-6 | 113 | 22,254 | 8,421,981 | 799,029 |

### Git

35 commits d06b4ba..80d1c70 · +1176 / −356 across 17 files

## Artifacts

**Committed (on main):**
- `orchestration_log/history/2026-07-09/session.md` — T-CAP narrative (frozen)
- `orchestration_log/history/2026-07-09/app-defect-catalog.md` — 7+1 actionable defects from capture suite
- `orchestration_log/history/2026-07-09/session.md` — this file (T-FIX + close)
- `orchestration_log/reference/conventions.md` — 2026-07-10 lessons appended
- `orchestration_log/reference/codebase_state.md` — 2026-07-10 and 2026-07-11 change blocks appended
- `orchestration_log/reference/deferred_items.md` — 2026-07-10 and 2026-07-11 entries appended

**Recon (gitignored, regenerable):**
- `orchestration_log/recon/2026-07-10/` — capture-inventory.md, app-state-space.md, capture-gap-design.md, verify-*.md, app-defect-catalog.md, reviews/
- `orchestration_log/recon/2026-07-11/` — component-propagation-audit.md, a10-spec-trace.md, verify-fixes-final.md, session_metrics.md, git_history.md, reviews/

**Generated (gitignored, regenerable):**
- `end2end/screenshots/` — regenerate via `bash scripts/isolated-capture.sh <suffix> visual` (standard pass) + `VISUAL_SPEC=tests/visual-audit-cohort.spec.ts bash scripts/isolated-capture.sh <suffix> visual` (cohort pass, requires `COHORT_CAPTURE=1` and sibling DB)

## Git History

```
2026-07-09 22:24:48 +0300 9c15ed1 feat(auth): add .auth-card layout + .btn[data-variant=link] (T02 CSS foundation)
2026-07-09 22:30:38 +0300 ee816fd feat(auth): page-frame/auth-card wrap, link back-buttons, OTP resend affordance
2026-07-09 22:36:26 +0300 812c7a8 fix(auth): route resend countdown label through parameterized i18n key
2026-07-09 22:42:49 +0300 b892ab0 refactor(auth): extract start_cooldown closure — one home for resend timer lifecycle
2026-07-09 23:04:26 +0300 f92bb6c fix(auth): cfg-gate resend timer body to client — set_interval panics during SSR
2026-07-10 18:25:22 +0300 a2bde49 fix(login): gate interval-handle storage and cleanup out of SSR build
2026-07-10 18:51:23 +0300 4df5d20 fix(login): gate resend dispatch call out of SSR build to silence startup panics
2026-07-10 19:03:35 +0300 a960710 test(visual-audit): clear cookies + assert phone-input visible before OTP capture
2026-07-10 19:08:28 +0300 07d2fc0 fix(login): move resend handler out of view! cfg gates; dual-cfg binding restores step visibility
2026-07-10 19:19:21 +0300 ab2ca3c fix(auth): stretch step wrappers to card width — D1 label alignment on mobile
2026-07-10 19:20:53 +0300 5cf9843 fix(auth): wrap disabled expr in braces — D2 '>' misparse as tag close in view!
2026-07-10 19:25:07 +0300 d74bce7 refactor(login): consolidate cooldown comments and restore hydration wait before otp-capture fill
2026-07-10 19:32:30 +0300 5da012b docs(login): comment load-bearing braces on disabled attribute in view!
2026-07-10 21:06:11 +0300 7629148 fix(capture): propagate playwright exit code + screenshot floor in isolated-capture.sh
2026-07-10 21:15:49 +0300 47226e8 fix(capture): clean screenshot marker in EXIT trap on all failure paths
2026-07-10 21:09:22 +0300 e0a508e fix(admin): expand cycle-viz viewBox to 600x560 and raise max-width to 480px
2026-07-10 21:17:52 +0300 443efdd fix(admin): match cycle-viz aspect-ratio to viewBox and inline geometry constants
2026-07-10 22:31:14 +0300 d06b4ba doc: session 2026-07-10 — auth SSR-abort close, T08 cycle-viz, harness exit fix, branch purge [skip ci]
2026-07-10 23:53:17 +0300 3ac07bf feat(visual-audit): stable naming, dark error states, gap fills, INDEX.md, pass B+C captures
2026-07-10 23:53:58 +0300 2c20e5c fix(visual-audit): use testid selectors for swap selects
2026-07-11 00:03:04 +0300 f6e6606 fix(visual-audit): distinct L7 resend-active capture and L11 used-code error after code consumption
2026-07-11 02:17:24 +0300 2556ab1 refactor(visual-audit): atomic recordScreenshot primitive, honest no-season guard and resend-cooldown naming
2026-07-10 23:52:08 +0300 80760b9 feat(capture): unit 7 — large-cohort cycle-viz pass with SQL seed and VISUAL_SPEC harness var
2026-07-10 23:58:49 +0300 8a009e5 fix(capture): gate cohort spec behind COHORT_CAPTURE opt-in and refuse seeding the shared samete DB
2026-07-11 00:06:14 +0300 786a54b fix(capture): make cohort seed idempotent with ON CONFLICT DO NOTHING guards
2026-07-11 02:28:57 +0300 5474fc2 fix(visual-audit): remove unreachable phone-error assertion
2026-07-11 02:38:57 +0300 85d30f4 fix(visual-audit): remove unreachable name-error capture (L15)
2026-07-11 02:49:47 +0300 1ee0114 fix(visual-audit): remove unreachable city-empty error capture (O2)
2026-07-11 02:52:18 +0300 88ccb64 fix(visual-audit): remove unreachable swap-error capture (A40)
2026-07-11 03:11:42 +0300 11709b3 fix(visual-audit): cohort afterAll merges its rows into INDEX.md idempotently
2026-07-11 03:12:03 +0300 b5c26da fix(visual-audit): capture receipt form before confirmReceipt() — not after
2026-07-11 03:12:18 +0300 0f45b7f fix(visual-audit): document H4a=H1 app-identical state — unlaunched season invisible to participants
2026-07-11 03:12:35 +0300 c71dfed fix(visual-audit): remove redundant L7 capture — identical to L5 due to cooldown
2026-07-11 03:12:49 +0300 83c07c3 fix(visual-audit): document A3 serial context — participants+used codes are honest state
2026-07-11 03:23:29 +0300 fd0501f doc(visual-audit): precise order-flip safety rationale in cohort INDEX merge comment
2026-07-11 03:30:05 +0300 d6a17f6 fix(visual-audit): distinct stateId for not-received receipt capture
2026-07-11 03:53:11 +0300 01878df doc: T-CAP close — capture-elevation record, app-defect catalog, reference updates [skip ci]
2026-07-11 12:22:19 +0300 7e12bc9 fix(404): render not-found page with page-frame/empty-state primitives
2026-07-11 12:25:04 +0300 589c7f2 fix(admin): route create-season errors to correct field; theme never invalid
2026-07-11 12:24:05 +0300 afd2dd3 fix(toast): move banner out of main into flex column to eliminate scroll-overlap
2026-07-11 12:30:59 +0300 356a547 fix(toast): drop unneeded z-index from in-flow toast container, sync layer-map doc
2026-07-11 12:35:40 +0300 0eb84ca docs(toast): sync doc comments to in-flow banner mechanism
2026-07-11 12:23:40 +0300 c3b7b27 fix(cycle-viz): quadrant-aware labels + two-line split eliminate 12-node collisions
2026-07-11 12:29:06 +0300 feed218 fix(cycle-viz): widen viewBox to 900x750 to prevent right-side label clipping
2026-07-11 12:25:03 +0300 501fd73 fix(ui): route info-link phone number color through --color-text
2026-07-11 12:25:34 +0300 26997be fix(ui): raise --color-panel-dark to 0.28 so alerts contrast admin-section cards in dark
2026-07-11 12:37:53 +0300 6646a2b doc(ui): sync panel-dark ratios; cap at 0.27 to keep alert error text AA
2026-07-11 12:59:31 +0300 2e001a5 fix(admin): use --color-text for em-dash in dark mode to fix A6 low-contrast
2026-07-11 13:27:57 +0300 27bc907 fix(css): replace testid selector with .inactive-marker semantic class
2026-07-11 13:28:44 +0300 4a9792f doc(guidance): fix z-index map (toast is in-flow), add .info-link spec entry
2026-07-11 13:47:17 +0300 b55259d doc: T-FIX close — 7 defects cleared + propagation audit record [skip ci]
2026-07-11 14:25:12 +0300 83b1f85 fix(i18n): finalize proposed uk copy after native review
2026-07-11 14:27:05 +0300 80d1c70 fix(i18n): align cancelled-season copy to ty-form voice
```
