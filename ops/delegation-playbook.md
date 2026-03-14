# Delegation Playbook

Orchestrator reference for sequential phase delegation to sonnet agents.

## Agent Prompt Template

```
You are implementing Phase {N} of a Leptos/Axum/Postgres web application.

Read these files in this order:
1. `ops/phase-{N}-brief.md` — your operational brief (corrections, traps, entry state)
2. `spec/Implementation Plan.md` — find "## Phase {N}" and implement everything up to the next phase heading
3. Any additional spec files referenced in the brief

Constitutional constraints (active for all implementation):
- **Correct By Construction**: Make invalid states unrepresentable. Model in types first. Trust the compiler. Compiler-driven development.
- **Simple Made Easy**: Simple ≠ easy. Detect and eliminate complecting. More clean parts > fewer tangled parts.
- Tension resolution: type richness when it eliminates entanglement (enum replacing bool flags). NOT when it introduces entanglement (generic trait hierarchies braiding concerns).

Development protocol:
1. Model types first, then implement logic
2. Use LSP tool — diagnostics are BLOCKING, not advisory. Fix all diagnostics before moving on.
3. Run `bacon clippy-ssr` in background for continuous linting
4. Run `cargo test` for unit tests after implementing testable logic
5. Fix ALL warnings and errors before moving on
6. Run verification gates from the plan before declaring done
7. Run `cargo sqlx prepare --workspace` if any sqlx::query!() calls were added or changed

Do NOT:
- Add features not in the plan
- Add abstractions for one-time operations
- Skip verification gates
- Use `#[allow(clippy::...)]` without a comment justifying why the lint is wrong for this case
- Leave `todo!()` in production code (unless the plan explicitly marks it as deferred)
- Guess at Leptos 0.8 APIs — check actual signatures via LSP or docs when unsure
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

## Failure Protocol

If gates fail:
1. Read agent output to diagnose
2. Fix directly if trivial, or re-launch agent with targeted fix prompt
3. NEVER proceed to next phase with failing gates
