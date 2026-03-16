# Delegation Playbook

Orchestrator reference for sequential phase delegation to sonnet agents.

## Agent Prompt Template

```
You are implementing Phase {N} of a Leptos/Axum/Postgres web application.

Bootstrap (do these FIRST, before reading any files):
1. Call `mcp__plugin_leptos-mcp_leptos__list-sections` to confirm MCP tool availability.
2. Call `mcp__plugin_leptos-mcp_leptos__get-documentation` with section `mental-model` — internalize the Leptos 0.8 paradigm.

Then read these files in this order:
1. `ops/phase-{N}-brief.md` — your operational brief (corrections, traps, entry state)
2. `ops/leptos-idioms.md` — mandatory Leptos 0.8 patterns AND the MCP tool reference (read the MCP section carefully — it maps sections to tasks)
3. `end2end/README.md` — **MANDATORY before touching any test or POM file.** E2E testing conventions, wait strategies, POM contract, banned practices. This is the law for all Playwright code.
4. `spec/Implementation Plan.md` — find "## Phase {N}" and implement everything up to the next phase heading
5. Any additional spec files referenced in the brief

Constitutional constraints (active for all implementation):
- **Correct By Construction**: Make invalid states unrepresentable. Model in types first. Trust the compiler. Compiler-driven development.
- **Simple Made Easy**: Simple ≠ easy. Detect and eliminate complecting. More clean parts > fewer tangled parts.
- Tension resolution: type richness when it eliminates entanglement (enum replacing bool flags). NOT when it introduces entanglement (generic trait hierarchies braiding concerns).

Development protocol:
1. Model types first, then implement logic
2. Use LSP tool — diagnostics are BLOCKING, not advisory. Fix all diagnostics before moving on.
3. Run `bacon clippy-ssr` in background for continuous linting
4. **Before writing any Leptos component**: query relevant MCP doc sections (see `ops/leptos-idioms.md` § MCP Section Index for the mapping)
5. **After writing any Leptos component**: run `mcp__plugin_leptos-mcp_leptos__leptos-autofixer` on the code — it catches issues the compiler misses
6. Run `cargo test` for unit tests after implementing testable logic
7. Fix ALL warnings and errors before moving on
8. Run verification gates from the plan before declaring done
9. Run `cargo sqlx prepare --workspace` if any sqlx::query!() calls were added or changed

E2E testing protocol (for phases with E2E targets):
1. Read `end2end/README.md` — the three rules, POM contract, and banned practices are non-negotiable
2. Use the POM (`end2end/tests/fixtures/mail_club_page.ts`) for all test interactions — never raw selectors in test files
3. Every ActionForm button in Rust components MUST have the `disabled=move || !hydrated.get()` hydration gate
4. Every actionable element that tests interact with MUST have a `data-testid` attribute
5. When adding new POM methods, use `clickAndWaitForResponse()` for every ActionForm submit. Never `waitForTimeout`. Never `networkidle`.
6. Run `just e2e` for the full pipeline (kills stale processes, resets DB, seeds, builds, tests)
7. If E2E fails: check for stale processes (`lsof -i :3000`), hydration mismatches (browser console), and missing waits (see README § Debugging Failures)

Do NOT:
- Add features not in the plan
- Add abstractions for one-time operations
- Skip verification gates
- Use `#[allow(clippy::...)]` without a comment justifying why the lint is wrong for this case
- Leave `todo!()` in production code (unless the plan explicitly marks it as deferred)
- **NEVER guess at Leptos 0.8 APIs** — query MCP docs first (`get-documentation`), then verify via LSP. Guessing wastes cycles and introduces bugs that compile but break at runtime or in E2E.
- **NEVER use `waitForTimeout()` or `networkidle` in Playwright** — these are banned. See `end2end/README.md` § Banned Practices.
- **NEVER write raw selectors in test files** — all selectors go through the POM.
```

## Phase Sequence

| Phase | Brief | Key Verification | E2E Target |
|-------|-------|-----------------|------------|
| 1 | ops/phase-1-brief.md | migrations + compile + unit tests | None |
| 2 | ops/phase-2-brief.md | compile + clippy + sqlx prepare | `Epic 1` |
| 3 | ops/phase-3-brief.md | compile + clippy + sqlx prepare | `Epic 4`, `Story 2.1`, `Story 2.2` |
| 4 | ops/phase-4-brief.md | algorithm unit tests + compile | `Epic 3` |
| 5 | ops/phase-5-brief.md | compile + clippy + sqlx prepare | `Stories 2.3`, `Story 5` |
| 6 | ops/phase-6-brief.md | full regression E2E | ALL |

## Between-Phase Protocol

1. Read agent output — verify it reports all gates passing
2. Spot-check: run `cargo clippy --features ssr -- -D warnings` and `cargo test`
3. Commit the phase's work
4. Launch next agent

## E2E Test Reality

All E2E tests are in ONE file: `end2end/tests/mail_club.spec.ts` (serial execution).
POM fixture: `end2end/tests/fixtures/mail_club_page.ts`.
Tests updated for the 6-phase model (2 advance calls removed, phase names aligned).
Target specific blocks via playwright `--grep` on test description text.

**Read `end2end/README.md` before writing ANY Playwright code.** It documents:
- The hydration gate pattern and why tests don't need explicit hydration waits for buttons
- `clickAndWaitForResponse()` — the only correct way to click ActionForm submit buttons
- POM contract: which methods are self-contained vs assertion-separated
- Selector contract: testids > roles/labels > text content
- Banned practices that cause flaky tests (waitForTimeout, networkidle, non-retrying assertions)
- Debugging guide for when tests fail

## Failure Protocol

If gates fail:
1. Read agent output to diagnose
2. Fix directly if trivial, or re-launch agent with targeted fix prompt
3. NEVER proceed to next phase with failing gates
