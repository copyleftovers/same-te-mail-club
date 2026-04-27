# Conventions

Last updated: 2026-04-20

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

## Forbidden Patterns

| Pattern | Why | Traced to |
|---------|-----|-----------|
| Claiming "all green" on single E2E run | Flakiness masks behind lucky runs | Session 2026-04-09: celebrated 58/58 on f8a3c41, but subsequent runs showed 1-3 failures |
| Pool config changes without metrics | Pool was never the bottleneck (max 4 connections) | Session 2026-04-09: $15 wasted on pool starvation hypothesis |
| `[patch.crates-io]` without running patched crate's own test suite | Byte-offset semantics broke silently | Session 2026-04-09: leptos_config regex patch broke config parsing |
| `waitForLoadState("load")` in POM | Waits for 14MB WASM download, intermittently exceeds timeout | Session 2026-04-09: every POM method with `"load"` eventually caused a timeout |
| Running `brew services start postgresql` | Shadows Docker Postgres on port 5432 | Session 2026-04-09: 3 failed E2E runs from brew/Docker collision |
| Router-wide `tokio::time::timeout` middleware on Axum SSR routes | Drops the SSR future mid-render; Leptos Suspense never resolves; client hangs until its own navigation timeout. Identical-budget timeouts on both server and client maximize the failure surface. | Session 2026-04-20: commit `3ad9b65` reverted in commit `1f4df2c` after CI exposed it. |

## Test Philosophy

- E2E tests are serial (shared DB state). Independent blocks (Epic 6, Logout) run separately.
- Static assets cached across tests via `cached-context.ts` fixture. Import `test`/`expect` from there, not `@playwright/test`.
- Pre-compressed WASM served via `precompress-and-test.sh`. `CompressionLayer` skips already-encoded responses.
- Verify with 2-3 consecutive green runs before declaring stability.
