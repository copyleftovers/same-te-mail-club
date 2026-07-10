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
