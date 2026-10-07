You are a CODE-QUALITY REVIEWER. First read your role definition and comply fully: /home/user/ryzhakar/claude-skills/dev-discipline/agents/code-quality-reviewer.md. Then read /home/user/same-te-mail-club/orchestration_log/recon/2026-10-04/fix/prompts/_code-quality-reviewer.md and obey it. You have a shell; run the gates you rely on.
Unit: U1 in /home/user/same-te-mail-club/orchestration_log/history/2026-10-04/plans/PLAN-blockers.md.
Worktree: /home/user/same-te-mail-club/.claude/worktrees/agent-aa77876d6f145827e
Branch: worktree-agent-aa77876d6f145827e
Diff range: 57f0109..4bccd33
Spec verdict (A3, PASS): /home/user/same-te-mail-club/orchestration_log/recon/2026-10-04/fix/reviews/spec-worktree-agent-aa77876d6f145827e-2330.md
Report file (A4): /home/user/same-te-mail-club/orchestration_log/recon/2026-10-04/fix/reviews/quality-worktree-agent-aa77876d6f145827e-0005.md
Read-path rule: reference docs under /home/user/same-te-mail-club/orchestration_log/recon/**, /tmp/claude-manifesto-repo/**, /home/user/ryzhakar/claude-skills/** are read at absolute paths; the A3/A4 paths above are exempt from re-rooting.
Context:
- wasm clippy on rustc 1.99 fails in untouched #[server] code; that is ENV scope.
- T0, the recursion fix, lands separately; set any limit locally only, never commit it.
- Build env: CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=3 SQLX_OFFLINE=true. Delete any target/ you create when done (disk is tight).
- Do not edit or commit in the worktree.
End your reply with the `Ready to merge:` line and the report path.
