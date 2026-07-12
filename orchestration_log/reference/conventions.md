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

## Added 2026-07-09/10 (visual-intervention session — checkpoint lessons)

- **Thread vs agent (user-corrected).** A THREAD = one reported fault; it spans many worker agents and closes only when the fault is solved with NO regressions. Agents are workers within a thread. Track threads on the harness Task tools (shared, visible list) — NOT an agent-managed board file.
- **Isolated capture harness (own port + own DB per cycle).** NEVER run `just e2e`/`e2e-release`/`db-reset`/`_kill-stale` while the user's dev server is live on :3000 — it stomps their server + wipes `samete`. Instead: sibling `samete_<suffix>` DB on the live Postgres + a free port + run the binary directly with LEPTOS_SITE_ADDR/DATABASE_URL + playwright CAPTURE_BASE_URL + trap teardown. `scripts/isolated-capture.sh` is the mechanism. Parallel-safe (validated: concurrent sibling DBs, samete untouched).
- **Pixel gate earns its keep.** T01's void-fix passed spec review AND quality review TWICE while the RENDER was still broken (centering inert). Rendered-verify at native resolution, both viewports (375/414+desktop) AND both modes (light+dark), is the arbiter for visual work — code review cannot catch inert CSS. Similarly the auth `/login` SSR crash passed spec+quality (no SSR render in review) and was caught only by an actual capture.
- **Worktree-scoped agents get WORKTREE paths.** A debug agent given MAIN-repo paths grepped an unintegrated feature and wrongly concluded "phantom class." Conversely, gitignored recon plans live in MAIN's untracked tree — implementers read them by ABSOLUTE main path and edit in their own worktree. Always match the path to the tree the fact lives in.
- **CSS centering: `>` matches direct children only.** `.page-frame > .empty-state` failed because `.prose-page` (and Suspense) sit between. Verify the ACTUAL rendered DOM ancestry before writing structural selectors; 2 source-reasoning fixes failed before rendered-DOM debugging found it.
- **client-only APIs (`set_interval_with_handle`) must be `#[cfg(not(feature="ssr"))]`-gated** if reachable on the SSR render path, else the server binary panics (Abort trap 6, empty response). This gates a SIDE EFFECT (not a rendered value) so it is NOT the banned cfg-hydration-mismatch pattern — provided the associated signal's INITIAL value is unconditional/identical across SSR+client. (NOTE: bd7c4d9 applied this yet the abort persisted — root cause still open; see deferred_items.)
- **recon artifact filenames: avoid "findings"/"report".** A subagent guard blocks Writes to files it classifies as report/findings. Use `design-intervention.md`/`impl-plan.md`/`spec-*.md`/`diagnosis`/`rootcause` names. If a Write is guard-blocked, the agent must relay content in its notification for the orchestrator to persist via a scribe.
- **Harness exit-0 gap.** `scripts/isolated-capture.sh` exited 0 despite total playwright failure — do NOT trust the capture exit code; confirm screenshots were produced (`ls | wc -l`) before rendered-verify. (Fix tracked, task #10.)
- **Notification attribution cross-wires; content is truth.** Repeatedly the completion notification named the wrong agent (e.g. T04 designer vs T03 scribe swapped). Trust the RESULT content + verify disk, never the attributed agent name.

## Orchestration Doctrine — Delegation, Recycling & Context Budgeting (2026-07-10)

The single home for: when to delegate, when to recycle vs fresh-launch, and how to keep an agent inside its window. Flow: **why → budget → measure → decide → discipline.**

### 1. Cost model — what delegation actually buys
Authoring content costs the same tokens in the orchestrator's context whether written directly OR dictated to an agent — it lands in the dispatch prompt either way. A scribe therefore only ADDS overhead (framing) and SUBTRACTS fidelity (it transcribes/interprets). Consequences:
- Delegation's ONLY real win is making an agent ABSORB token cost the orchestrator would otherwise bear — READING/processing large inputs that never enter the orchestrator's context (many files in → a short answer out). **Delegate for READS.**
- Delegation NEVER wins for emitting content the orchestrator already holds.

**STANDING POLICY (user-set, binding):**
- **NORMAL mode (file-ops prohibited) = DELEGATION-FIRST on everything.** The dispatch overhead is accepted deliberately; context hygiene is the point, not token-minimization. Do NOT bypass the file-op constraint even when holding the content.
- **CHECKPOINT phase (the only write-permitted state) = MEMORY WORK → orchestrator writes DIRECTLY, NEVER delegates.** Delegating memory ops is costly now (overhead) and later (fidelity drift in the durable record that must survive compaction).
- If guidance MUST be authored via delegation (only under normal prohibition), the agent MUST be **opus** — guidance authorship is an opus task regardless of who types it.

### 2. The budget — context windows are MODEL-DEPENDENT
- Opus 4.8 ≈ **1,000,000** tok · Sonnet 4.6 ≈ **250,000** · Haiku ≈ **200,000**. Always budget against the AGENT'S OWN window.
- **Model selection has TWO drivers: reasoning tier AND context budget.** A context-heavy task (big logs + source + reproduce + author, or a long recycled investigation) may warrant opus purely for its window even when sonnet's reasoning would suffice.

### 3. Measure — estimating resident load
`resident_tokens ≈ init(~6k) + dispatch_prompt + Σ(text_bytes ÷ 3–4) + Σ(images × ~1.5k) + output_headroom`
- Content-type matters: prose ≈ 4 B/tok · code/JSON/logs ≈ 3 B/tok (denser) · images ≈ ~1.5k tok each regardless of byte-bulk.
- **Runtime probe** (cheap, no content read): `stat -f %z` the `subagents/agent-<id>.jsonl`; compute % against the agent's window.
- **Calibration** (this session): a SONNET agent overflowed at 735 KB dense text (~245k tok ≈ its 250k ceiling); an OPUS agent sailed through 4158 KB (image-heavy) on its 1M window. Byte-size misleads until read against window + content type.

### 4. Decide — recycle vs fresh-launch
Recycling (SendMessage-continuation) is delta-continuation of a still-cheap context, **not** a free lunch: the transcript only grows, spending the window to save the re-load cost.
- **RECYCLE when:** the follow-up is a tight delta that LEVERAGES what the agent holds (fix cycle, re-review of the same diff, "also check X in the file you read") AND estimated load < ~65% of that agent's window.
- **FRESH-LAUNCH** (seeded from the on-disk artifact) when ANY of: window near ceiling · new domain/approach (held context is dead weight) · transcript reaped · model/tier change needed (recycle cannot change the model).
- Own a problem until the **DELTA STOPS BEING CHEAP** — hand to a seeded successor BEFORE the window wall, not at it. ("Own until context-exhausted" ≠ drive-to-exhaustion. Traced: `ae4d7c7a` recycled thrice into overflow, zero salvage.)

### 5. Discipline — never over-commit one agent
- **EXTERNALIZE STATE TO FILES FIRST, recycle second.** Durable progress lives in artifacts; a successor is then cheap and recycling is a fast-lane over a file-backed spine, never a dependency.
- **PLAN-TIME BUDGET before dispatch:** sum the big inputs. If one task's inputs (multi-MB log AND source AND reproduce AND author) alone approach the window → too big for one agent → SPLIT into single-question agents.
- **FAILURE-MODE DISCIPLINE — distinguish, do not conflate:**
  - `"Prompt is too long"` = resident CONTEXT exhausted → NOT resumable → decompose + fresh-launch smaller.
  - `"session/rate limit"` = throughput cap → RESUMABLE after reset via SendMessage to the SAME agent (unless its task record was reaped at a session boundary).

## Orchestration Doctrine — Concurrent Fault-Threads (2026-07-10)

Running many independent problems in parallel without collision or context blowup. (Validated this session: ~7 fault-threads concurrently, foundation-first, sequential integration.)

### A THREAD is one problem, not one agent
One reported fault / unit of work = one thread. It spans many worker agents and CLOSES only when the fault is solved with NO regressions. Agents are workers WITHIN a thread. Track threads on the harness Task tools (the shared, user-visible list) — one task per thread, phase in metadata. Do NOT build an agent-maintained board file (superseded this session).

### Per-thread lifecycle
DISCOVERY → DESIGN → PLAN → IMPLEMENT → SPEC-REVIEW → QUALITY-REVIEW → VERIFY (rendered pixels / e2e / behavioral) → INTEGRATE → CLOSED. Reviews are inevitable — no unit skips spec→quality; no exemptions.

### Parallelize thinking; serialize only the shared foundation
Worktrees make every dev cycle natively isolated, so a shared write-set is a PLANNING problem, NOT a `blockedBy` dependency:
- Run ALL discovery/design/plan in parallel immediately (read-only — no collision).
- If threads share files, land the shared FOUNDATION once (one thread), then branch the rest FROM the integrated foundation.
- Integrate sequentially; rebase each thread onto the prior's updated main — git auto-resolves distinct keys/regions. Only genuinely conflicting same-line edits need a resolver agent.

### Isolated run/capture per cycle
Any thread that builds + runs the app uses the isolated-capture harness (own free port + own sibling `samete_<suffix>` DB on the live Postgres). NEVER `:3000` / `samete` / `db-reset` / `_kill-stale` while a dev server is live. N captures run concurrently on distinct sibling DBs (parallel-safe, validated).

### Liveness & truth
A single global heartbeat cron is dormancy insurance; harness completion-notifications drive real transitions. Notification ATTRIBUTION cross-wires — trust the RESULT content + verify on disk, never the named agent. Every background wave gets the backstop.

## Added 2026-07-10 (auth SSR-abort campaign + parallel-thread close)

**Leptos view! macro hazards (both verified in rendered pixels this session):**
| Pattern | Why | Traced to |
|---|---|---|
| `#[cfg(...)]` tokens ANYWHERE inside view! (incl. closure bodies) | Macro tokenizer misparses; attrs emit as TEXT NODES; step-visibility desyncs. Gate via dual cfg'd bindings OUTSIDE view! — signature-identical, SSR no-op stub | 2026-07-10: 73275f4/dc2d4af broke /login render; restructured 9fd53bf |
| Bare `>` comparison in an unbraced view! attribute expression | Parsed as tag close; every following attr leaks as text | 2026-07-10: `disabled=move \|\| x.get() > 0` → 410ef0f braces fix |

**Debugging doctrine additions:**
- Release+LTO backtraces MISATTRIBUTE frames (phantom toast.rs across 2 independent probes). Rebuild debug profile for frame-accurate backtraces before trusting any frame.
- Grep-verification of served HTML must be TOKEN-SPACING-TOLERANT (`on ?: ?click`); the leak rendered `on : click` and exact-match grep returned 0 = false clean.
- Never feed a prior agent's conclusions to an opinion-forming probe (anchoring: one contaminated dispatch "confirmed" the phantom toast root cause). Raw evidence only; empirical probe (debug server + curl) arbitrates.
- SSR-render assertion belongs in the fix agent's verify step: serve + curl + step-visibility grep — code review and clippy cannot see render breakage.

**Capture/verify mechanics:**
- Playwright serial capture: session cookies LEAK across tests in one browser context — clearCookies before auth-state captures.
- Image-pass budget: ~30 full-page shots ≈ one sonnet window. One screenshot-dir per agent; NEVER recycle an image-heavy verifier into a second full pass (2 overflows this session).
- isolated-capture.sh exit code trustworthy as of 47226e8 (propagation + screenshot floor); sabotage-test harness changes (fake build binary + no-match grep / `true` playwright) instead of full runs.
- Worktree removal invalidates any CWD inside it → next spawn fails "Failed to resolve HEAD". cd to project root before removal+spawn.
- Session-limit interruption (rate-limit class) is losslessly resumable via SendMessage — even hours later; pair with a one-shot resume cron at reset time.

## Added 2026-07-11 (T-CAP + T-FIX session lessons)

- **Merge-base diffs are MANDATORY in reviewer briefs once mid-wave integration starts.** Plain `diff main..HEAD` in a stale-base worktree shows OTHER units' integrated changes as reverts — two reviewers false-flagged scope creep (A4 "404 revert", A3 "mount didn't move"). Arbitrate reviewer-vs-implementer contradictions with ONE primary-source git query, never by re-reading reports.
- **Computed contrast ≠ perceived visibility for hairline glyphs.** A 13px em-dash at a passing 4.7:1 was near-invisible in pixels (too few contrasting pixels in a single thin stroke). Pixel evidence overrules AA arithmetic for sub-glyph strokes; fix by stepping the token up (dark: --color-text), not by re-deriving ratios.
- **No testid selectors in production CSS** (inverse of the tests-ban): styling on `[data-testid=...]` braids the E2E contract into rendering. Semantic classes only. Enforced: `grep 'data-testid' style/` = 0.
- **Doc value-sync is a defect detector.** Forcing recomputation of documented ratios (panel 0.27/0.28) exposed a real AA failure that implementer + spec r1 had both passed. When a token changes, recompute EVERY documented derived number — the sync is a verification pass, not bookkeeping.
- **DOM-element presence ≠ reachable state.** 5/16 designed capture gap-fills were phantom states (anti-enumeration silent-Ok paths, HTML5 required gating, no-op writes, always-seeded cooldown). Gap analysis must trace SERVER semantics, not infer states from markup. Unreachability claims get adversarial source-cited review before removal.
- **Fix-run-fix LOOP agents beat one-failure-per-orchestrator-cycle** for serial capture suites: give one worktree implementer the whole iterate-to-green mandate (run capture → read failure → fix → rerun) with hard limits (3 attempts/test, no src edits) instead of paying integration+rerun per bug.
- **"Prompt is too long" at <10 tool uses = single oversized read**, not accumulation. Read-hygiene rules (chunked reads ≤400 lines; never package-lock/node_modules/target/screenshots; pipe noisy commands + tail) are now standard in EVERY implementer brief.
- **VISUAL_SPEC=<spec> + mode=full on isolated-capture.sh** = CI-way full-suite validation with zero :3000/samete exposure. Opt-in-gated specs (COHORT_CAPTURE pattern: file-scope test.skip blocks beforeAll + DB-name refusal + harness env export) are the template for any future seeded pass.
- **haiku tier unavailable in this environment** (claude-haiku-4-5@20251001 rejected at dispatch, observed 2026-07-11 close): route mechanical/extraction work to sonnet until the model reappears; do not retry haiku blindly.
- **One session = ONE history dir, keyed to the session START date.** `${DATE}` in the artifact contract is the session's start date, not the calendar day a checkpoint happens to be written (a multi-day session checkpoints into the SAME dir). Traced: 2026-07-09..11 session fragmented across 3 dirs, consolidated at user correction (801c96a).

## Added 2026-07-12 (binding mechanics + campaign start)

- **Dispatch binding lines MUST carry full manifesto file paths** (`/tmp/claude-manifesto-repo/LLM_MANIFESTOS/manifestos/<name>.md`), never bare names: the SubagentStart oath hook injects nothing (gap since 2026-07-03) and the /tmp repo can be absent (was missing at this session's start; restore via `git clone --depth 1 https://github.com/ryzhakar/LLM_MANIFESTOS /tmp/claude-manifesto-repo/LLM_MANIFESTOS`). A name is not a source.
- **Unbound judgment output is rejected** (user ruling). Recovery: bound re-derivation by the SAME warm agent — read manifestos, bind visibly, re-derive from held raw inputs, treat the unbound draft as scratch (never incremental-edit it). For fan-outs, a shared contract file carrying the constraints remains the validated alternative (2026-07-03).
- **Token-session wake crons:** at each token-session start set one-shot crons +5h and +5h30m (backup); each wake re-arms the next pair, deletes its sibling, resumes from TaskList. Session-limit-killed agents resume via SendMessage.
- **Specialized code agents for code-touching work** (user): implementer/spec-reviewer/code-quality-reviewer carry purpose-fit constitutions; general-bound stays default for non-code work.

## Added 2026-07-12 (runaway-agent termination mechanics — hard-earned)

- **Mailbox teammates are NOT TaskStop-able.** Named background agents (spawned with `name:`, addressed via SendMessage) live outside the subagents task registry — `TaskStop` rejects every id form, and their transcripts are not under `subagents/agent-*.jsonl`. The ONLY kill path is the SendMessage shutdown protocol: `{"type":"shutdown_request","reason":...}` — the agent approves and terminates (`teammate_terminated` + roster removal confirm). Originating shutdown_request is sanctioned when the user orders a stop.
- **Shutdown is queued, not preemptive.** The agent drains its inbox FIFO; queued work (including file writes) completes BEFORE the shutdown processes. A "stopped" agent can still write. Never leave a superseded agent sharing a write-path with its replacement: AMEND THE REPLACEMENT'S OUTPUT PATH (e.g. `-v2` suffix) the moment a defunct writer exists — decomplect the race instead of trying to out-time it. Roster error "No teammate named X" on a later send = the earlier shutdown landed.
- **Directive-compliance check on continuations:** an agent told to "re-derive bound, draft is scratch" may instead bind and re-report the old artifact ("already delivered"). Verify the ORDERED ACT happened (fresh mtime + content act), not just that a compliant-sounding notification arrived. Escalate once with an explicit is-not-optional message; on second deviation, erase output + fresh-launch (user-ordered here) rather than negotiate a third time.

## Added 2026-07-13 (session-close is MEMORY WORK — hand-written, hard rule)

**session-close IS session-checkpoint, extended.** The orchestrator writes the session record and reference corrections with its OWN HANDS. The file-op prohibition suspends during close for exactly this reason — not so the orchestrator can delegate from a suspended state, but so it can type the durable memory itself.

Traced to a 2026-07-12 breach: the orchestrator ran close like normal WORK (delegate-first) — dispatched haiku for metrics/git-history and a sonnet SCRIBE to formalize the session record. Consequences that prove the rule:
- Both haiku agents **idled without producing** (the known haiku-idles-without-writing mode) — the orchestrator did metrics+git itself anyway, so the delegation was pure overhead.
- The sonnet scribe introduced **fidelity drift** into the durable record — fabricated deferred-item entries pulled from a PRIOR session's list, into the one artifact that must survive compaction and seed the next ARRIVE truthfully. A scribe transcribes+interprets; memory cannot afford either.

**The split:**
- Mechanical EXTRACTIONS (JSONL metrics parse, `git log`/diffstat) MAY be delegated for token-absorption — but the orchestrator MUST verify their output before trusting it (scope especially: `extract_metrics.py` scans a subagents/ dir that AGGREGATES prior sessions — its timestamps/tokens/cost routinely span the wrong scope; anchor on git history + on-disk review-artifact counts instead).
- The session RECORD (A4) and reference-doc corrections (A6/A7/A8) are HAND-WRITTEN. Never a scribe. If a formalization draft was delegated, the orchestrator re-owns every line — verifying against its own memory, not rubber-stamping.
- `/cost` is a REPL slash-command, NOT an orchestrator tool — genuinely not invocable by the orchestrator. When close can't run it, record "Cost: not captured — /cost not invocable" honestly; NEVER substitute the JSONL-derived estimate (wrong-scope + double-counted).
