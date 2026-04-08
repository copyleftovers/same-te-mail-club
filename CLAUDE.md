
# NON-NEGOTIABLE INITIALISATION

Upon starting, check this index of manifestos first: 'https://raw.githubusercontent.com/ryzhakar/LLM_MANIFESTOS/refs/heads/main/README.md'.
Use `curl` for efficient pulling of raw text data, not the fetch tool.
Use the 'manifest-oath' skill to swear to any of those manifestos when prompted.
This project being a rust project and striving to be idiomatic, use the 'correctness-by-construction' manifesto by default.
If continuing a session after context compaction - reswear to the active manifestos anew.
If swearing to more than 1 manifesto, figure out their interplay and interdependencies early: hierarchy, governance, conflict resolution, interference, amplification.
Upon figuring out the graph of manifesto interdependence and multiactivation, write it down in the most natural way accessible to you.

# DELEGATION
Delegate often and well. You must have a very solid and unexcusable reason to do anything yourself. Unles you have one, you MUST delegate.
Use the /agentic-delegation skill constantly.
Generally, you would want to use simpler models for any subagents, unless there's a reason to do otherwise.
For any given delegation, you need to make an explicit decision whether to retain the conversation or not.
Rely on externalized context for delegation as a first-class citizen, prefering it to the handing-down the conversation history whenever possible.
Context, instructions and preferences are externalized as manifestos, plans, artifacts, operational notes, etc.

Plans must survive handoff to agents who lack your context. Use defensive-planning skill to do so for longer sequences of implementation steps.

If anything can be delegated and done in parallel, you MUST use multiple parallel agents.
One of the workflows where this pattern lends itself beautifully is objective fault analysis based on each of the active manifestos by separate agents.

---

# CLAUDE NOTES

## Active Manifestos

Bind to all three on session start (treat as unified framework):
1. **Correct By Construction**
2. **Simple Made Easy**
3. **First Principles — Break the Mold**

## Guidance

- @guidance/dev-protocol.md — feedback loop, compiler rules, LSP, TDD, unit vs E2E boundary (**binding**)
- @guidance/leptos-idioms.md — Leptos 0.8 patterns (mandatory for all component work)
- @guidance/debugging-policy.md — E2E failure delegation, long-running command rules
- @guidance/design-system.md — palette, typography, components, density, accessibility (**binding**)
- @guidance/frontend-protocol.md — Tailwind v4 setup, CSS architecture, banned patterns (**binding**)
- @end2end/README.md — E2E testing guide, POM contract, wait strategies (**binding**)

## Product Design

→ `spec/product/` — vision, personas, product decisions
→ `spec/technical/` — architecture, data model, user stories

## Key Commands

| Command | Purpose |
|---------|---------|
| `just dev` | Start dev server (hot reload) |
| `just test` | Run unit tests |
| `just clippy` | Run clippy (SSR) |
| `just e2e` | Run Playwright E2E tests |
| `just check` | Full validation: fmt + clippy + test |
| `just db-reset` | Drop, create, migrate database |
| `just prepare` | Generate sqlx offline query data |
| `bacon` | Continuous clippy. Keys: `s` SSR, `h` hydrate, `t` tests |

**Environment:** `just e2e` requires `source .env.example` (or `.env`) for `DATABASE_URL`. Do not run `just` targets without it.

## QA Automation

The `/qa-run` skill orchestrates E2E test execution and locator healing. The existing suite lives in `end2end/` — bridge files at `tests/seed.spec.ts` and `.playwright/project-config.md` map the skill to this layout. All bridge artifacts are gitignored.

**Pipeline phases:** PLAN and GENERATE are skipped (suite already exists). Only EXECUTE (run + classify failures) and HEAL (fix broken locators) run.

**JSON reporter:** `end2end/playwright.config.ts` outputs `results.json` for structured agent parsing. The output file is gitignored.

## E2E Pitfalls (Learned)

- **Login race:** `login()` POM method must `waitForLoadState("load")` after the OTP verify redirect. Without it, subsequent `page.goto()` fires mid-redirect and SSR responses hang (Suspense never resolves).
- **Redundant navigation:** `goHome()` skips `page.goto("/")` if already on `/`. The dev WASM bundle is ~14MB; redundant full reloads intermittently exceed the 15s `navigationTimeout`.
- **Serial cascade:** All 58 tests run in one `test.describe.serial` block. If any test fails, all subsequent tests skip. Epic 6 (account management) is truly independent and could be split into a separate serial block to avoid cascade — not yet done.
