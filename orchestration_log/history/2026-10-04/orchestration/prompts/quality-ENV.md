You are a CODE-QUALITY REVIEWER. First read your role definition and comply fully: /home/user/ryzhakar/claude-skills/dev-discipline/agents/code-quality-reviewer.md. Then read /home/user/same-te-mail-club/orchestration_log/recon/2026-10-04/fix/prompts/_code-quality-reviewer.md and obey it. You have a shell; run the gates you rely on.
Unit: ENV in /home/user/same-te-mail-club/orchestration_log/history/2026-10-04/plans/PLAN-blockers.md, INCLUDING the "ENV addendum (2026-10-07, PRB-102)".
Worktree: /home/user/same-te-mail-club/.claude/worktrees/agent-a9394f7e60a701d45
Branch: worktree-agent-a9394f7e60a701d45
Diff range: ae111c8..3adddfc. The ENV commits are 799e247, a5fed31, c82b4eb and 3adddfc; 1ce283e only merges the base branch in.
Spec verdict (A3, PASS): /home/user/same-te-mail-club/orchestration_log/recon/2026-10-04/fix/reviews/spec-worktree-agent-a9394f7e60a701d45-1000.md
Report file (A4): /home/user/same-te-mail-club/orchestration_log/recon/2026-10-04/fix/reviews/quality-worktree-agent-a9394f7e60a701d45-1150.md
Read-path rule: reference docs under /home/user/same-te-mail-club/orchestration_log/recon/**, /tmp/claude-manifesto-repo/**, /home/user/ryzhakar/claude-skills/** are read at absolute paths; the A3/A4 paths above are exempt from re-rooting.
Focus:
- Bootstrap script: robustness, idempotence, and failure modes on a fresh container and on macOS.
- The SessionStart hook: it must be safe, fast enough, and not block the session on network failure.
- CI changes: the rustup toolchain install path, and the cargo-audit pin.
- The crate-level lint allow in src/lib.rs.
- .gitignore exceptions.
Build env: CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=3 SQLX_OFFLINE=true. Do not edit or commit in the worktree. Delete any target/ you create.
End your reply with the `Ready to merge:` line and the report path.
