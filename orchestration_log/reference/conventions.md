# Conventions

Last updated: 2026-07-04

## Model Tier Overrides

| Task type | Default tier | Override | Rationale |
|-----------|-------------|----------|-----------|
| POM/E2E fixes | sonnet (implementer) | — | Needs reasoning about race conditions, wait strategies |
| E2E investigation | sonnet (explore) | — | Needs to trace server function → SSR → Suspense chains |
| E2E debugging / suite flakiness investigation | sonnet (multiple parallel) | Required | History-informed; mandatory speculative-parallel per agentic-delegation. Single-agent investigation has historically missed cross-cutting causes (see Phase A 2026-04-20). |
| Pool metrics / binary analysis | haiku | — | Mechanical: run command, read output, report |
| CLAUDE.md / docs updates | opus (agent) | Required | Instructions for agents must be written by opus |
| Internet research | sonnet | — | Needs judgment to assess credibility of sources |
| leptos_config patching | sonnet (implementer) | — | Needs source-code-level reasoning about byte offsets |

**Lesson:** The user explicitly requires opus for anything touching agent instructions (CLAUDE.md, guidance docs). This is a hard override, not a suggestion. Traced to: session where sonnet agent was interrupted for docs work.

## Dispatch Rules

1. **E2E debugging is ALWAYS delegated.** Per `guidance/debugging-policy.md`. Orchestrator never reads screenshots, traces server function logic, or runs multiple E2E cycles.
2. **Long commands use `run_in_background` + redirect to file.** Never pipe through `| tail` or `| head`. Read output file separately.
3. **Source `.env.example` before any `just` target.** Docker Postgres requires `DATABASE_URL` set. Failure mode: `role "samete" does not exist` or `Connection refused`.
4. **Verify Postgres is Docker, not brew.** `docker compose ps` first. If brew Postgres is running on 5432, it shadows Docker's port mapping.

## Review Protocol

- spec-reviewer after implementer reports DONE (per dev-orchestration)
- code-quality-reviewer after spec passes
- For E2E POM changes: verification is running the suite, not code review. Run 2-3x to confirm non-flakiness.

## Visual Development Pipeline (added 2026-06-25)

The binding loop for UI/visual work. Externalizes discovery to artifacts so the loop never re-pays discovery cost; gates on rendered pixels, not diffs; recycles agents so opus stays affordable.

```
IMAGE CAPTURE (initial) → ONE-TIME DISCOVERY (→ discovery artifacts)
  → ┌ FIX LOOP (reuse agents, until all pass) ───────────────────────┐
    │ IMPLEMENTER (opus, reused) ◄──── back up on ANY reviewer FAIL ──┐│
    │   → IMAGE CAPTURE (post-fix, worktree)                          ││
    │   → SPEC-REVIEWER (opus, reused; loads discovery artifacts on   ││
    │       first run AND every re-init) ── FAIL ─────────────────────┤│
    │   → CODE-QUALITY-REVIEWER (sonnet, reused) ── FAIL ─────────────┘│
    └─────────────────────────────────────────────────────────────────┘
  → INTEGRATE (orchestrator ff/cherry-pick on spec PASS + quality PASS) → push HELD
```

Rules (binding):
- **Image capture = worktree e2e.** Screenshot the worktree build (`just e2e` in the worktree → `screenshots/{desktop,mobile,sections}/`). Once initially, then after EVERY implementer commit — reviewers judge the render, never the diff alone. `npm ci` in the worktree's `end2end/` on first capture (node_modules is gitignored → absent in a fresh worktree; persists for later captures).
- **One-time discovery.** A deep visual sweep runs ONCE (orchestrator's prerogative on agent count/tiers — e.g. opus fan-out by concern). Product = durable per-concern **discovery artifacts** (`recon/<date>/reviews/ui-*-r*.md`) = the externalized definition of "immaculate." The loop READS them; it never re-runs discovery.
- **Tiers:** discovery = opus (one-time). implementer = opus. spec-reviewer = opus. code-quality-reviewer = sonnet. Cost is controlled by RECYCLING (SendMessage continuation = delta-only), NOT by tier-downgrading.
- **Spec-reviewer is the heavy gate:** reads code diff + discovery artifacts + screenshots; passes only if spec-compliant AND render-immaculate against the artifact checklist. It MUST load the discovery artifacts on its first run AND on every re-init (context overflow / reaped transcript → fresh launch loses memory; reloading restores the checklist). Files are the durable spine; agent memory is the fast lane.
- **Code-quality-reviewer (sonnet)** runs only after spec PASS — narrower code-hygiene pass.
- **Reuse agents via SendMessage** across loop iterations (delta = the specific failing findings). Fresh launch ONLY when a transcript is reaped or a tier change is needed; a fresh agent reloads the discovery artifacts.
- **Loop until spec PASS AND quality PASS**, then orchestrator integrates (ff/cherry-pick to main). Push HELD unless the user explicitly asks.
- **Orchestrator never reads source/screenshots in its own context.** Verdicts arrive via agent notifications (each reviewer ends its return with `VERDICT: …` / `Verdict:`). Reading a small gating verdict file is the only sanctioned exception, and even that is preferably delegated.
- **Every background dispatch gets a cron backstop** (see Notification & Liveness Protocol).

Caveats learned this session:
- `SendMessage` is the continuation tool but deferred-tool availability flickers (it was absent mid-session, then reappeared). When absent, OR when an agent's transcript is "cleaned up" (SendMessage fails), fall back to a fresh agent that reloads the artifacts.
- **Section-screenshot crops are unreliable for clip/overflow** — the crop-window has a DPR offset that fabricates clipping (a real mobile-stepper "clip" was an artifact; a real one was missed). Judge clips/overflow on FULL-PAGE shots ONLY; reserve section crops for detail (typography/component close-ups).
- **Automate the mechanical visual checks** (clip / overflow / overprint / element-past-viewport at 375px + desktop) as Playwright assertions → deterministic, free per run; reserve agent eyes for aesthetic judgment (cohesion/hierarchy/typography). Direction, not yet built — the invite-table overprint and stepper clip are exactly the class that should be assertion-caught.

## Added 2026-06-25 (close: recycle + CI lessons)

- **RECYCLE context-holding agents via SendMessage; NEVER fresh-launch a substitute when a valuable resumable agent holds the context.** A fresh-opus gate was launched instead of recycling — wasteful re-reads of diff+artifacts+screenshots. Recycling costs only the delta message. (User's hard correction.)
- **Local `just clippy` masks sqlx OFFLINE cache staleness.** Local clippy validates `query!`/`query_as!` against the LIVE DB (DATABASE_URL set); CI runs `SQLX_OFFLINE=true` against `.sqlx/`. ANY query change — INCLUDING whitespace/indentation in an SQL string literal — changes the sqlx hash and REQUIRES `cargo sqlx prepare --workspace -- --features ssr`. Before pushing after touching a query, verify the CI way: `SQLX_OFFLINE=true cargo clippy --no-default-features --features ssr`. Traced: `47984a7` SQL re-indent → CI run 28171533086 fail at Check→clippy(SSR).
- **CI babysitting via gh:** `gh run list --branch main --commit <full-sha> --json databaseId,status,conclusion,workflowName` to match a specific push's run. `gh run list` does NOT support `--arg` (jq-var passthrough fails). Recurring ~4-min cron; success → PushNotification + delete cron; failure → failing job name + delegate a sonnet log diagnosis (`gh run view <id> --log-failed`); NEVER read full CI logs in orchestrator context.
- **A single recycled opus gate** can subsume spec + visual + code-quality in one pass when equipped with the discovery artifacts (externalized concern checklist) + diff + screenshots — fewer agents, full robustness. Refines the Visual Development Pipeline above.

## Forbidden Patterns

| Pattern | Why | Traced to |
|---------|-----|-----------|
| Claiming "all green" on single E2E run | Flakiness masks behind lucky runs | Session 2026-04-09: celebrated 58/58 on f8a3c41, but subsequent runs showed 1-3 failures |
| Pool config changes without metrics | Pool was never the bottleneck (max 4 connections) | Session 2026-04-09: $15 wasted on pool starvation hypothesis |
| `[patch.crates-io]` without running patched crate's own test suite | Byte-offset semantics broke silently | Session 2026-04-09: leptos_config regex patch broke config parsing |
| `waitForLoadState("load")` in POM | Waits for 14MB WASM download, intermittently exceeds timeout | Session 2026-04-09: every POM method with `"load"` eventually caused a timeout |
| `waitForLoadState("domcontentloaded")` after redirects in POM | Racy: if DOMContentLoaded fires before the call, it waits for the NEXT one (which never comes) → 30s timeout. Use `expect(page).toHaveURL()` / `not.toHaveURL()` (auto-retrying, race-free) instead. | Session 2026-06-20: 4 failed fix attempts before root-causing the race. Affects login, logout, completeOnboarding, selfRegister, advanceSeason. |
| `waitForURL` with `waitUntil: 'domcontentloaded'` | Same race as standalone `waitForLoadState` — the `waitUntil` option has identical semantics. | Session 2026-06-20: attempted as fix, failed identically. |
| Adding `value=""` to Leptos ActionForm inputs for hydration stability | Leptos hydration resets `.value` to match the attribute — erases Playwright-filled values. Absence of `value` attribute is better: Leptos has nothing to restore. | Session 2026-06-20: onboarding branch input. Added `value=""`, tests failed worse. Reverted. |
| Scapegoating the machine for test failures | Dev-mode E2E failures are real bugs, not environmental noise. Fix them. | Session 2026-06-20: user correction. |
| Running `brew services start postgresql` | Shadows Docker Postgres on port 5432 | Session 2026-04-09: 3 failed E2E runs from brew/Docker collision |
| Router-wide `tokio::time::timeout` middleware on Axum SSR routes | Drops the SSR future mid-render; Leptos Suspense never resolves; client hangs until its own navigation timeout. Identical-budget timeouts on both server and client maximize the failure surface. | Session 2026-04-20: commit `3ad9b65` reverted in commit `1f4df2c` after CI exposed it. |
| Orchestrator reading/editing `.rs` files or using Edit tool on source | Orchestrator context is the most expensive resource. Source file reads and edits are agent work. When debug agents fail, dispatch narrower agents — never intervene directly. | Session 2026-04-29: orchestrator read login.rs 10+ times and used Edit 8+ times to debug hydration/cookie issues. |
| `#[cfg(feature = "ssr")]` for values that must match across hydration | SSR and WASM branches produce different initial values, causing hydration mismatch. Use query params, shared constants, or Resource (with stable fallback) instead. | Session 2026-04-29: `is_pending` was true on SSR, false on client — invite-code-step disappeared during hydration. |
| ActionForm for server fns that set HttpOnly cookies | Fetch API's Set-Cookie handling differs from native POST. Cookies set via `ResponseOptions` may not apply to ActionForm fetch responses. Use native `<form method="post">` instead. | Session 2026-04-29: register_with_code session cookie never set via ActionForm; switched to native POST. |
| LEFT JOIN columns without `"column?"` suffix in sqlx query_as! | sqlx's offline cache may mark LEFT JOIN columns as non-nullable. At runtime, NULL values cause decode errors. Always use `"column?"` to force nullable decode. | Session 2026-04-29: list_invite_codes failed with "unexpected null; try decoding as Option" for redeemer_name. |
| `toBeEnabled()` guards on `goHome()`/`goToDashboard()` | When WASM fails to init (intermittent dev-mode), explicit `toBeEnabled` waits 15-30s then hard-fails. Playwright's `click()` auto-retry is more resilient — it retries continuously with backoff. | Session 2026-06-22: goHome logout-button 30s wait caused worse flakiness than the original `<main>` visible check. |
| Removing `page.goto("/admin")` after admin login in tests | Login redirect to /admin is not deterministic — SSR Suspense may not resolve admin state before the test proceeds. Explicit goto forces fresh SSR round-trip. | Session 2026-06-22: visual-audit test 207 failed consistently after goto removal. Restored. |
| `cargo sqlx prepare` without `--features ssr` | Queries exist only in the SSR feature gate. Bare prepare finds 0 queries, deletes the entire cache. Always: `cargo sqlx prepare --workspace -- --features ssr` | Session 2026-06-22: empty .sqlx/ after prepare. |
| Stripping `content-encoding` from cached-context.ts headers | Hypothesis was wrong — Chromium handles the Content-Encoding: br mismatch on route.fulfill() more gracefully than receiving raw bytes with no encoding header. The "fix" made hydration worse. | Session 2026-06-22: 4 consecutive failures after applying the encoding fix. Reverted. |
| **Agents running `git merge`/`git push` to main** | Integration is the ORCHESTRATOR's job (cherry-pick/ff after both reviews pass). An agent merged a fix to main itself, outside protocol AND before its review completed. | Session 2026-06-25: an implementer/investigation agent created merge commit `ddacc91` on main. Content was clean, but it bypassed the gate. Agents commit to their worktree ONLY. |
| **Width-tuning a layout that doesn't fit (form-blindness)** | If content is clipped/overflowing/crammed/squeezed, the FORM is usually wrong for that viewport, not the dimensions. | Session 2026-06-25: 8 rounds (nowrap → min-width → sibling-cap → table-layout:fixed) failed to fit a 5-col table in 65ch. Question the form first (see `guidance/ui-review-prompt.md`). |
| **Per-fix re-verify as a substitute for holistic review** | Checking that specific fixes landed rubber-stamps "immaculate" while the page is holistically janky. | Session 2026-06-25: per-group agents passed admin repeatedly; user saw it as "janky as hell." Use harsh holistic design judgment at native resolution. |
| **`table-layout: auto` + `min-width` to reserve a column under a width cap** | Auto-layout ignores `min-width` as a hard reservation when the table is `width:100%` inside a capped container (65ch); the wrapper's `overflow-x:auto` never scrolls (table never exceeds it) — columns squeeze/clip. `table-layout:fixed` honors widths but does NOT clip overflow (too-narrow columns overprint neighbors). | Session 2026-06-25: revoke-pill clip, 8 attempts. A 5-col table does not fit 65ch. |

## Notification & Liveness Protocol (added 2026-06-25)

- **A notification firing IS the completion signal (truth).** Only the attribution/content cross-wires — one task-id was observed reporting three different agents' outputs. On ANY notification, go check the relevant **verdict/marker FILES** for the real content; never trust the notification's claimed agent/result.
- **ALWAYS set a cron backstop per background dispatch** (the agentic-delegation rule, non-negotiable). An 8-hour dormancy occurred from skipping a backstop on a "quick" reviewer whose completion notification never routed.
- **Don't over-engineer liveness.** Marker-file + bg-bash watcher scaffolding is unnecessary; notification-react + a recurring cron backstop (that polls files and carries the next steps) is the mechanism.
- **Rendered pixels are the arbiter for visual work.** Static code/spec review passed the toast/heading/table defects; only the screenshots caught them. Always re-verify on the actual rendered screenshots, at native resolution (thumbnails/loose crops fabricate clips and hide real ones).
- **New shared reviewer prompt:** `guidance/ui-review-prompt.md` — generic, harsh, holistic, form-first UI-review method. Supply the target per dispatch; use this over the older `/tmp/visual-reverify-spec.md`.
- **Screenshot dir hygiene:** the visual-audit spec writes screenshots but never cleans; orphans accumulate across runs. After each reshoot, `find end2end/screenshots -name '*.png' -mmin +3 -delete` to keep only the fresh set.
- **db-reset blocker:** an open external SQL client (DataGrip) holds Postgres connections and blocks `sqlx database drop`. `just db-reset` now prepends `pg_terminate_backend`.

## Test Philosophy

- `just e2e` runs release binary (471KB WASM). `just e2e-dev` for dev mode (14MB, flaky).
- E2E tests are serial (shared DB state). Independent blocks (Epic 6, Logout) run separately.
- Static assets cached across tests via `cached-context.ts` fixture. Import `test`/`expect` from there, not `@playwright/test`.
- Pre-compressed WASM served via `precompress-and-test.sh`. `CompressionLayer` skips already-encoded responses.
- Verify with 3 consecutive green runs before declaring stability.
- `waitForLoadState` is banned in all forms after redirects. Use auto-retrying `toHaveURL` / `not.toHaveURL` assertions.
- Every review chain runs sequentially within a unit (spec → quality). Multiple units' chains can run in parallel.

- E2E tests are serial (shared DB state). Independent blocks (Epic 6, Logout) run separately.
- Static assets cached across tests via `cached-context.ts` fixture. Import `test`/`expect` from there, not `@playwright/test`.
- Pre-compressed WASM served via `precompress-and-test.sh`. `CompressionLayer` skips already-encoded responses.
- Verify with 3 consecutive green runs on CI before declaring stability. Dev-mode green is necessary but not sufficient.
- `waitForLoadState` is banned in all forms after redirects. Use auto-retrying `toHaveURL` / `not.toHaveURL` assertions.
- Every review chain runs sequentially within a unit (spec → quality). Multiple units' chains can run in parallel.

## Added 2026-07-03 (harsh visual-review method — validated)

**Two-axis visual-discovery swath** (worked excellently: 36 sonnet agents + 1 opus synthesis, ~20 min wall, ~150 raw findings → 24 actionable fix-units):
- **Axis A — per-page-state holistic** (one sonnet per state × both viewports). Catches within-page jank (hierarchy, rhythm, clip/overflow, state completeness).
- **Axis B — per-concern cross-page** (pills/buttons, badges, fields, typography, color/surface/dark, spacing/density, logo/grain, whole-product cohesion). **The concern axis owns "N treatments for M functions" defects** — a per-page reviewer accepts local pills; a concern reviewer collecting EVERY instance across all pages can't. This axis found the user's exact complaint.
- **Deterministic static component-inventory feeds Axis B** — component drift is a clustering problem over computed styles/classes, not pure eyeball. Page agents (Axis A) get NO inventory → independent pixel judgment → corroboration with the inventory stays meaningful (page agents independently reproduced inventory finds = strong signal).
- **Externalize ONE shared REVIEW-CONTRACT.md** (harsh anti-rubber-stamp contract + fixed parseable output schema + severity scale + rubric-doc pointers + screenshot paths). Dispatches stay thin (axis + target + a hint). simple-made-easy: one spec, N thin dispatches.
- **Anti-rubber-stamp mechanism that WORKED:** "assume the target is broken; 'looks fine' is a review failure not a pass; zero findings requires element-by-element proof in a Coverage section." Every agent found real MAJORs; agents honestly hedged "needs computed verification" (contrast/touch-px) instead of fabricating. Directly counters the 2026-06-25 rubber-stamp failure.
- **Spot-check batch 1** (inventory + ~5 diverse states) before the full fan-out — validates the shared contract cheaply (6 agents) before committing the other 31.
- **Group the synthesized catalog by FIX-UNIT, not by page** — deliverable is a systemic backlog; one systemic fix clears many scattered page findings.
- **Reuse screenshots when HEAD is doc-only over the last code commit** — pixels are the current render; a rebuild is byte-identical. Verify no code commit since capture (`git log`).
- Output contract per agent: return ONE line (VERDICT + file path); orchestrator reads the FILES (small verdict/findings md — the sanctioned read exception), never the screenshots/source. Opus synthesis reads all N files (its context, not the orchestrator's).

**Foreground-intent / async-actual + cron backstop:** Agent spawns went async ("running… via mailbox") despite pure-fan-out foreground intent. Per background wave, set a recurring ~7-min cron backstop that polls output files + carries the next steps + self-deletes on completion. Notification = completion signal; files = truth; cron = dormancy insurance (2026-06-25). Tally completions from notifications; verify file counts on disk before the gated next step (synthesis).

**Manifesto subagent-oath hook GAP (observed 2026-07-03):** spawned `general-bound` agents bind to 0 elements — SubagentStart is not injecting `.manifestos.yaml` oaths. Until fixed, carry the binding constraints IN the dispatch/contract (the review contract did — output was fully harsh/terse/first-principles regardless). Do NOT rely on the hook for spawned-agent binding.

## Added 2026-07-04 (HARD lessons — review-discipline breach + recovery)

**Forbidden (traced to this session's breach — user's furious callout):**
| Pattern | Why | Traced to |
|---|---|---|
| **Integrate-on-build-verify / skipping the spec→quality chain** | dev-orchestration invariant: review is inevitable — EVERY unit spec→quality→integrate. Integrated 4 commits unreviewed + one on a spec-FAIL, chasing a deadline. | 2026-07-04: cb4f3db/7ccee77/1c71787 unreviewed; 1bb710f on FAIL |
| **Rationalizing a review exemption** ("render is the real gate" / "test code needs no review" / "the FAIL is only LE1") | Each is a discretionary bypass of a NON-discretionary gate. Integrate ONLY on both spec PASS + quality PASS as verdict FILES. A FAIL never integrates. Test/capture code gets the SAME chain. | 2026-07-04 |
| **Fresh-launching instead of reusing context-holders** | Reuse via SendMessage (delta-only); fresh only if reaped or tier-change. | 2026-07-04 (restates 2026-06-25) |
| **Reading verdict/source files in orchestrator context** | Wastes the expensive resource ("wasting my money"). Reviewers RETURN the verdict line in their reply; cross-wired → delegate a cheap reader agent; NEVER Read the file yourself. | 2026-07-04 |

**Trust = a verifiable mechanism, not a promise.** The review artifacts (verdict files + commit ordering) are the auditable trail; the orchestrator HOLDS the gate itself (not the user babysitting). A leash makes the orchestrator useless.

**Visual-fix loop that WORKED (post-restoration):** capture (complete matrix incl. dark via `emulateMedia`, error-state, focus, long-content) → complete review (concern agents + opus synth → catalog) → per-unit implement(worktree)→spec(reuse)→quality(reuse)→integrate-on-both-green → re-capture (regenerate on FIXED code; doubles as the e2e regression gate) → rendered re-verify (RECYCLE the finding agents; CLEARED/RESIDUAL on fresh pixels) → SURFACE residuals (don't tune blindly). DELEGATE all reads.

**Leptos 0.8 gotcha:** `attr:aria-invalid` (any `attr:`-prefixed attribute) on a NATIVE element emits a literal attribute NAMED `attr:...` — the prefix leaks into the name. Use BARE `aria-invalid` on native elements. This silently broke EVERY error border app-wide.

## Added 2026-07-04 (fix-everything phase lessons)

- **A roundup/list agent that reads a catalog with pre-integration STATUS tags will re-list ALREADY-FIXED items.** Force reconciliation against CURRENT source (grep the actual defect signature), and give every fix implementer a verify-then-fix mandate (fix only what's genuinely broken now; no-op the rest) — so exact list-accuracy is non-critical.
- **Own design/product decisions with sane defaults; do NOT pepper the user with design questions.** (User: "what's with all the nannying.") Reserve questions for genuine forks with no competent default (e.g. final marketing copy). Surface the chosen default; let them veto.
- **Mode-invariant `--color-badge-*` tokens for badges.** Semantic aliases like `--color-error`/`--color-success` REASSIGN (lighten) in dark → white-on-fill drops below AA. A dedicated badge-fill token that does NOT reassign keeps text-on-fill contrast identical in both modes. New status-color families: success→green, info/awaiting→muted-blue, attention/open→amber, terminal-neutral→gray, error→dedicated red, pending→amber.
- **Some implementer agents complete work then idle WITHOUT committing** (observed repeatedly with the CSS implementer). Bake "COMMIT + report the SHA" as an explicit final step; if it stalls, dispatch a throwaway committer agent to build-verify + commit the worktree (do NOT commit it yourself).
- **Reviewer completion notifications can cross-wire (content not routed, only an idle ping).** The verdict is in the verdict FILE. Delegate a cheap reader agent to return just the `Verdict:` line — NEVER read the file in orchestrator context.
- **CSS+logic coupled fixes must be one unit.** A CSS-only half (e.g. a toast slide-out keyframe with no JS/Rust to trigger it) is inert dead code — and worse if commented as if wired. Do the keyframe + the trigger logic together.

## Added 2026-07-04 (Wave 2 lessons — session 7c7c3839)

**Fold-cheap-Minor-wins-before-integrate policy.** Rule: if a Minor fix is newly-introduced by the unit AND cheap with an already-present pattern → fold into the unit's commit set (save a review cycle). Skip if: pre-existing in the codebase (different diff scope), cosmetic without a clear pattern to reuse, or a reviewer explicitly declined it. Folds executed this wave: drop-clone + None-field-routing + np-number-error testid (unit D), named timing constants + uniform try_get_value (unit F), i18n existing-address key (unit C).

**Rendered re-verify catches what code/spec review cannot.** This wave caught three defects ONLY in rendered pixels — never in code or spec review:
1. OTP field-level width cap: error text wrapped to 4 ragged lines (rv-participant → fix 0422a77).
2. Invite-card mobile balloon: desktop left/right split retained at 375px (rv-admin → fix c3433f5).
3. Login empty-state heading demotion: `<p>` instead of `<h1>` for a page's only heading (spec-w2-home first review, visible in pixel hierarchy).
ALWAYS re-verify rendered pixels at native resolution, BOTH viewports (375px + desktop) AND BOTH color modes (light + dark). Dark shots live in `screenshots/dark-desktop` and `screenshots/dark-mobile`. Orchestrator pointing at light-only was corrected mid-session at user challenge.

**`.btn + .btn` gap does not cross `<form>` boundaries.** The adjacent-sibling combinator can't reach a button OUTSIDE the form. Use a token-based top margin (`mt-(--density-space-sm)`) on the standalone button instead.

**Width cap belongs on the input, not the field wrapper.** Setting `max-width` on `.field` or `.field-input`'s parent means the `.field-error` inherits the cap and wraps inside a narrow column. Cap the `<input>` element (or `.field-input`) only; let `.field` and `.field-error` stay full-width.

**Mobile card stacking requires an explicit breakpoint override.** A desktop left/right split (flex-row layout) does NOT degrade to a single column at 375px unless a `@media (max-width:639px)` block explicitly restacks. The same breakpoint and idiom used by `.data-table` (mobile card degradation) must be applied consistently to ANY desktop split-column card. No auto-degradation.

**Premature idle pattern.** An implementer or recap agent reports idle while a detached `just e2e-release` or `cargo leptos build` keeps running as a background process. The idle ping is not a completion signal. Monitor the log file + `pgrep` to confirm the process has exited before treating the output as final.

**SendMessage stall vs stale agent.** A reviewer that doesn't respond to a re-review message is stale (transcript cleaned up or reaped). Fresh-launch a substitute ONLY when the agent doesn't acknowledge the message. If the agent responds but gives a slow verdict, wait — reusing context is cheaper than a full reload.

**verify-then-fix in every implementer brief.** Absorbs stale-defect-list drift: if a defect was already fixed by a prior commit, the implementer skips it cleanly rather than producing a no-op or regressing. The brief should always say "verify the defect is genuinely present before fixing; if already fixed, report it as no-op."

**Docs-only close commit uses `[skip ci]`.** The session-close commit (orchestration_log/ markdown only) needs no CI; append `[skip ci]` to its one-line message so a docs-only push doesn't trigger a redundant full CI run (the code CI already ran on the preceding code push).

**One writer per living ref-doc per close.** The orchestrator hand-appended a conventions section while the close-writer agent independently extended the SAME file → duplicate 2026-07-04 sections, both committed, then a dedup commit to clean up. Either the orchestrator writes a ref-doc section OR a close agent does — not both concurrently. If unavoidable, grep the dated section headers before committing.

**Pre-commit fixer hooks abort the first commit.** The `trim trailing whitespace` / `fix end of files` hooks modify the staged file and fail the commit ("files were modified by this hook"); re-`git add` + re-commit lands it. Budget one retry after any doc append.

**Forbidden (Wave 2 additions):**
| Pattern | Why | Traced to |
|---|---|---|
| Pointing rendered-re-verify agents at light-only screenshots when the change touches color | Dark mode has distinct token assignments; a color change that passes light may fail dark. `screenshots/dark-desktop` and `screenshots/dark-mobile` always exist after a full capture. | 2026-07-04 Wave 2: orchestrator missed dark shots; user challenged |
| Capping `.field` width to constrain an input | `.field-error` inherits the cap → error text wraps in a narrow column. Cap `.field-input` or the `<input>` element only. | 2026-07-04: login OTP 4-line error wrap, fix 0422a77 |
| Desktop card split layout without a mobile breakpoint override | No auto-degradation at 375px; cards balloon/rag. Explicit `@media (max-width:639px)` required. | 2026-07-04: invite-card mobile balloon, fix c3433f5 |
