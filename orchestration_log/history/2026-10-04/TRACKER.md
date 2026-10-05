# Solution Tracker — 2026-10-04 readiness campaign

Problems live in `PROBLEMS.md` (same dir). This file tracks only solutions: which unit fixes what, where the work is, and its state. Update on every state change; commit + push immediately.

## Resume from scratch (orchestrator dead)
1. Branch `claude/loving-johnson-7l8hn5` = integration branch. Plans: `plans/PLAN-blockers.md`, `plans/PLAN-product.md`, `plans/PLAN-findings.md` (unit specs; every unit is a concurrent lane).
2. Contracts/bindings (tracked copies): `orchestration/prompts/_implementer.md`, `_spec-reviewer.md`, `_code-quality-reviewer.md`, `_parallelism.md`, `lane-<UNIT>.md`; `orchestration/binding/<role>.md`. Live originals sit in gitignored `orchestration_log/recon/2026-10-04/` — if absent, copy these back there (lane prompts reference those absolute paths).
3. Manifestos: `git clone --depth 1 https://github.com/ryzhakar/LLM_MANIFESTOS /tmp/claude-manifesto-repo/LLM_MANIFESTOS`. Skills/agent defs: `git clone --depth 1 https://github.com/ryzhakar/claude-skills /home/user/ryzhakar/claude-skills` (then `claude plugin marketplace add` + install manifesto, orchestration, dev-discipline, agent-conduct, product-craft, qa-automation).
4. Cycle per unit (no exceptions): dev-discipline:implementer (worktree; dispatch = lane file) -> dev-discipline:spec-reviewer (verdict file) -> dev-discipline:code-quality-reviewer (report file, `Ready to merge:`) -> integrate (merge into branch, union `locales/uk.json` keys, `cargo sqlx prepare --workspace -- --features ssr`, fmt/clippy ssr+wasm/test gates, one-line commit, push, remove worktree).
5. Unit status below. A unit "implementing" whose worktree is gone = relaunch from its lane file. Branch names `worktree-agent-<id>`; check `git worktree list`, `git branch -a | grep worktree-agent`.
6. Environment gotchas until unit ENV lands: cargo-leptos install; Playwright browser path (`/opt/pw-browsers`); tailwind glibc binary; local-only `#![recursion_limit="256"]` until T0 lands; build env `CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0`; disk: dedupe `hardlink -c */target/*/deps` across worktrees; max 20 concurrent subagents.

## Recovery state (2026-10-05 20:40 UTC)
All implementers were killed by a session limit at ~09:20 UTC. Their work is preserved on this branch as patches in `wip/` (`INDEX.txt` lists per worktree: branch, base, commit count, uncommitted size; `<id>.commits.patch` = committed work, `<id>.uncommitted.patch` = working-tree diff). Apply with `git am` / `git apply` onto the listed base to recover a lane without the remote machine. Worktree target/ dirs were deleted (disk). Order of recovery: land U3+U4 (quality reviews resumed), then resume the lanes (SendMessage to the same agent ids — resume, not relaunch).

## Units
| Unit | Plan | Problems (PRB) | State | Branch / worktree | Spec | Quality | Integrated |
|---|---|---|---|---|---|---|---|
| T0 view-type depth root fix + CI build/E2E guards | blockers | TBD | implementing | worktree-agent-a18ac82e5b9ac2a54 | - | - | - |
| T1 server panic=unwind | blockers | TBD | implementing | worktree-agent-a4d2369a7e8477b86 | - | - | - |
| ENV reproducible toolchain + SessionStart hook | blockers | TBD | implementing | worktree-agent-a9394f7e60a701d45 | - | - | - |
| U1 registration tickets (OTP bypass) | blockers | TBD | implementing | worktree-agent-aa77876d6f145827e | - | - | - |
| U2 test-mode confinement | blockers | TBD | implementing | worktree-agent-ae294272a7d5f6210 (base + U3) | - | - | - |
| U3 first-admin bootstrap | blockers | TBD | quality review | worktree-agent-a1fe18df1d59b4c14 @ fd94520 | PASS | pending | - |
| U4 swap position exchange | blockers | TBD | quality review | worktree-agent-a46c4c68440f8d984 @ 066ebf2 | PASS | pending | - |
| A1 attr:aria-busy leak | blockers | TBD | implementing | worktree-agent-aa08619b87efefc0d | - | - | - |
| P-COPY copy fixes | product | TBD | implementing | worktree-agent-ad7158d9abfc86219 | - | - | - |
| P-S SMS site link | product | TBD | implementing | worktree-agent-ac57401dbc39b3668 | - | - | - |
| P-V real E2E assertions | product | TBD | implementing | worktree-agent-ab7ca384dc0b558ce | - | - | - |
| P-H1 participation home states | product | TBD | implementing | worktree-agent-a17d3d793043b76c0 | - | - | - |
| P-R organizer roster | product | TBD | implementing | worktree-agent-a82424857366c0f92 | - | - | - |
| P-F forwarding requests | product | TBD | implementing | worktree-agent-a36ce1f5f8b4f3c4b (base + U4) | - | - | - |
| P-H2 delivery card persists | product | TBD | implementing | worktree-agent-a8f818e1aa0d43eaa | - | - | - |
| P-H3 branch editable | product | TBD | implementing | worktree-agent-adb44670e29cec0d1 | - | - | - |
| P-M1 meetup storage + form | product | TBD | implementing | worktree-agent-ab0d6b0268101da57 | - | - | - |
| P-M2 meetup + timeline on home | product | TBD | implementing | worktree-agent-a961a61a4015cfb42 | - | - | - |
| P-DOC badge doc sync | product | TBD | not launched | - | - | - | - |
| F-* (17 code lanes + DOC-OPUS + ORCH) | findings | TBD | planned, not launched | - | - | - | - |

## Integration log
| Time (UTC) | Unit | Merge commit | Gates |
|---|---|---|---|
