You are a SPEC REVIEWER. First read your role definition and comply fully: /home/user/ryzhakar/claude-skills/dev-discipline/agents/spec-reviewer.md (if absent, find spec-reviewer.md under /root/.claude/plugins/cache/). Then read /home/user/same-te-mail-club/orchestration_log/recon/2026-10-04/fix/prompts/_spec-reviewer.md and obey it. You HAVE a shell: run the gates yourself instead of trusting the report.
Unit: T1 in /home/user/same-te-mail-club/orchestration_log/history/2026-10-04/plans/PLAN-blockers.md (absolute path).
Worktree: /home/user/same-te-mail-club/.claude/worktrees/agent-a4d2369a7e8477b86
Branch: worktree-agent-a4d2369a7e8477b86
Diff range: 57f0109..a6bb8be
Verdict file: /home/user/same-te-mail-club/orchestration_log/recon/2026-10-04/fix/reviews/spec-worktree-agent-a4d2369a7e8477b86-2340.md
Implementer report notes (context only, do not trust):
- RED (abort) and GREEN (unwind) stress runs reported.
- E2E: 3 of 4 runs green; run 2 failed at visual-audit.spec.ts:800 with a strict-mode violation, inactive-status matched 2 rows.
  - The implementer guessed leftover DB state in a reused sibling DB. A flake is a missing wait or a real bug, never noise: root-cause it, and judge whether it is caused by T1 or pre-existing.
- The T0 guard script is not on this branch; T0 lands separately.
- The wasm brotli size is 533602, the same before and after; the plan's 460-490k range may be stale.
Build env: CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=3 SQLX_OFFLINE=true. Postgres: pg_ctlcluster 16 main start; use a sibling DB, never `samete`. Do not edit or commit anything in the worktree; revert any local-only edits.
End your reply with the verdict line and the verdict file path.
