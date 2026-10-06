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
All implementers were killed by a session limit at ~09:20 UTC. Their work is preserved on this branch as patches in `wip/` (`INDEX.txt` lists per worktree: branch, base, commit count, uncommitted size; `<id>.commits.patch` = committed work, `<id>.uncommitted.patch` = working-tree diff). Apply with `git am` / `git apply` onto the listed base to recover a lane without the remote machine. Worktree target/ dirs were deleted (disk). Lane agents resumed 20:45 UTC. A background loop snapshots wip/ patches every 15 min onto this branch. Order of recovery: land U3+U4 (quality reviews resumed), then resume the lanes (SendMessage to the same agent ids — resume, not relaunch).

## Recovery log
- 2026-10-06 00:50 UTC: machine restored from an old disk snapshot (local branch at 5360af1, worktrees at ~09:20 state). Branch fast-forwarded from origin; every worktree rebuilt from wip/ patches of 2026-10-05 21:15 (old state kept in git stash entries `pre-restore-<id>`); Postgres restarted; all 18 agents resumed; loop scripts now tracked in orchestration/scripts/.

- 2026-10-06 10:50 UTC: all 18 agents resumed; all killed by the weekly limit (reset 17:00 UTC).
- 2026-10-06 17:55 UTC: resume policy changed (owner): SMALL BATCHES of 3, each batch finishes (implement -> spec -> quality -> merge) before the next. Order: U3 fix + U4 minors + T0; then T1, U1, U2; then ENV, A1, P lanes; then P-DOC; then F lanes. Loop scripts live at orchestration_log/history/2026-10-04/orchestration/scripts/.
- 2026-10-06 18:00 UTC: batch 1 = U3-fix (DONE, quality re-review sent to ae5b62914242ed4a1), U4-minors, T0; T1 resumed into U3's freed slot.
- 2026-10-06 18:25 UTC: U4-minors DONE @781241f, quality re-review sent to aae408a3826c4c153. U1 resumed into the slot. rustc drifted to 1.99 and wasm clippy fails in untouched code (32 errors); this is ENV's scope.
- 2026-10-06 19:00 UTC: U3 merged (gates: fmt, ssr clippy, ssr tests 92/0) + pushed; U4 minors merged (same gates green) + pushed. Running: T0, T1, U1.
- 2026-10-06 ~20:00 UTC: T0, T1 and U1 were killed by a session limit (reset 22:40 UTC). The worker restarted. 22:50 UTC: Postgres and the loops restarted, T0/T1/U1 resumed.
- 2026-10-06 23:05 UTC: T0 DONE_WITH_CONCERNS @4d000e7 (no recursion_limit; E2E 116/2/0 once; sabotage gates not run). Spec review dispatched (a4164fc74e49884c1, prompt orchestration/prompts/spec-T0.md). T1 and U1 still running.
- 2026-10-06 23:15 UTC: T0 spec FAIL. Missing: LoginPage .into_any(), the InviteCodeStep and NameStep extractions, sabotage gates, 3x E2E and pasted gate output; the stale CI comment must go; the 2 extra components need justifying. The findings went back to the T0 implementer. Next spec reviewer: give it a shell (general-purpose + role file); the dev-discipline:spec-reviewer type has no Bash.
- 2026-10-06 23:30 UTC: U1 DONE_WITH_CONCERNS @4bccd33 (3x E2E 120/2/0 reported; the curl gate was skipped). Spec review (general-purpose + role file, with shell) dispatched: a2360fe04bb536092, prompt spec-U1.md. U2 is held so no more than 3 agents run at once.

## Units
| Unit | Plan | Problems (PRB) | State | Branch / worktree | Spec | Quality | Integrated |
|---|---|---|---|---|---|---|---|
| 2026-10-05 20:45 | U4 | 823905e | reviewer gates green in worktree; post-merge gates running |
| T0 view-type depth root fix + CI build/E2E guards | blockers | TBD | implementing | worktree-agent-a18ac82e5b9ac2a54 | - | - | - |
| T1 server panic=unwind | blockers | TBD | implementing | worktree-agent-a4d2369a7e8477b86 | - | - | - |
| ENV reproducible toolchain + SessionStart hook | blockers | TBD | implementing | worktree-agent-a9394f7e60a701d45 | - | - | - |
| U1 registration tickets (OTP bypass) | blockers | TBD | implementing | worktree-agent-aa77876d6f145827e | - | - | - |
| U2 test-mode confinement | blockers | TBD | implementing | worktree-agent-ae294272a7d5f6210 (base + U3) | - | - | - |
| U3 first-admin bootstrap | blockers | TBD | INTEGRATED | worktree-agent-a1fe18df1d59b4c14 @ 067b9bd | PASS | Yes | merged 2026-10-06 |
| U4 swap position exchange | blockers | TBD | INTEGRATED; minors fixed @781241f, quality Yes, merged 2026-10-06 | worktree-agent-a46c4c68440f8d984 @ 066ebf2 | PASS | Yes (4 minors) | 823905e |
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
