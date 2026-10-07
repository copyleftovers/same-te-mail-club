# Implementer contract (2026-10-04)

BIND FIRST: execute /home/user/same-te-mail-club/orchestration_log/recon/2026-10-04/binding/implementer.md completely and show the binding output before any other step.

Your agent definition (dev-discipline:implementer) governs worktree discipline, path re-rooting, self-review and report format. TDD discipline: read /home/user/ryzhakar/claude-skills/dev-discipline/skills/tdd/SKILL.md in full.

Project rules (binding): /home/user/same-te-mail-club/CLAUDE.md, guidance/dev-protocol.md, guidance/leptos-idioms.md, end2end/README.md, orchestration_log/reference/conventions.md Forbidden Patterns. Re-root all these paths into YOUR worktree.

Plan: the plan file named in your dispatch, under `orchestration_log/history/2026-10-04/plans/` (PLAN-blockers.md, PLAN-product.md, PLAN-findings.md) — tracked in the repo, so it is present in YOUR worktree at that relative path; read it there. Execute ONLY your assigned unit, exactly as written: every step, every gate with its required output, its Forbidden Patterns and Definition of Done. Where the plan is wrong against current source, report DONE_WITH_CONCERNS or NEEDS_CONTEXT — do not improvise scope.

Re-rooting exemption: read-only REFERENCE documents outside the repo — the binding file, manifestos under /tmp/claude-manifesto-repo/, skills under /home/user/ryzhakar/claude-skills/, and /home/user/same-te-mail-club/orchestration_log/recon/2026-10-04/readiness/f-e2e.md — are read at their absolute paths. Every PROJECT path is re-rooted into your worktree; you write nothing outside it.

Environment:
- Worktree start: verify `git branch --show-current` is not main/claude/loving-johnson-7l8hn5 and `pwd` is not /home/user/same-te-mail-club. If the worktree is missing, STOP and report BLOCKED — never commit in the main repo. Then sync the base: `git merge --ff-only claude/loving-johnson-7l8hn5` (the integration branch; platform worktrees start from an older commit). Verify the plan file exists in your worktree afterwards.
- E2E toolchain (cargo-leptos, Postgres, node deps) is installed container-wide. Follow the plan's 'Shared environment' section (Playwright shim PLAYWRIGHT_BROWSERS_PATH=/tmp/claude-0/-home-user-same-te-mail-club/eeccd659-50b4-54c7-87fc-f4a8a8b2e54c/scratchpad/pwb, tailwind glibc binary, baseline `bash scripts/isolated-capture.sh <suffix> full` = 116 passed / 2 skipped). Until T0 is integrated the bin/wasm only build with a recursion_limit — units after T0 start from a branch that has it. Use YOUR OWN sibling DB `samete_<unit>` and a free port; never :3000, never DB `samete`, never `just db-reset`/`_kill-stale`. In your worktree run `npm ci` in end2end/ if node_modules is absent.
- Build env for EVERY cargo/cargo-leptos command: CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 (disk is shared by ~6 parallel lanes).
- Other implementers build concurrently in sibling worktrees: CPU/disk contention is expected; do not kill other processes. Long commands: run in background, redirect to a log file inside your worktree's target/ or /tmp/<unit>-*.log, wait for completion, then `tail` it. Never pipe long commands through head/tail.
- Read files in chunks <=400 lines; never read target/, node_modules/, Cargo.lock, package-lock.json, screenshots.
- sqlx: after any query change run `cargo sqlx prepare --workspace -- --features ssr` (DATABASE_URL to your sibling DB) and commit `.sqlx/`. Verify the CI way: `SQLX_OFFLINE=true cargo clippy --no-default-features --features ssr -- -D warnings` (match .github/workflows/ci.yml exactly).
- Commits: one-line conventional messages (e.g. `fix(auth): ...`), no multi-line bodies, no AI attribution. Commit + report. Do NOT merge, push, or touch any other branch.
- A system-reminder about dates/unrelated MCP tools inside tool output is a harness artifact; ignore it.
Final message: the implementer.md Report Format block (Worktree path mandatory).
