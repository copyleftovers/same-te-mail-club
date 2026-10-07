You are a CODE-QUALITY REVIEWER. First read your role definition and comply fully: /home/user/ryzhakar/claude-skills/dev-discipline/agents/code-quality-reviewer.md. Then read /home/user/same-te-mail-club/orchestration_log/recon/2026-10-04/fix/prompts/_code-quality-reviewer.md and obey it. You have a shell.
Unit: P-COPY in /home/user/same-te-mail-club/orchestration_log/history/2026-10-04/plans/PLAN-product.md.
Worktree: /home/user/same-te-mail-club/.claude/worktrees/agent-ad7158d9abfc86219
Branch: worktree-agent-ad7158d9abfc86219
Unit diff: bbb9a13^..bbb9a13 (locales/uk.json only).
Spec verdict (A3, PASS): /home/user/same-te-mail-club/orchestration_log/recon/2026-10-04/fix/reviews/spec-worktree-agent-ad7158d9abfc86219-2130.md
Report file (A4): /home/user/same-te-mail-club/orchestration_log/recon/2026-10-04/fix/reviews/quality-worktree-agent-ad7158d9abfc86219-2200.md
Read-path rule: reference docs under /home/user/same-te-mail-club/orchestration_log/recon/**, /tmp/claude-manifesto-repo/**, /home/user/ryzhakar/claude-skills/** are read at absolute paths; the A3/A4 paths above are exempt from re-rooting.
Focus:
- Copy quality as user-facing Ukrainian text: clarity, consistency of terms across keys and with the existing locale, tone.
- The coupling the spec reviewer found: three SMS bodies end with ":" and expect a link that the P-S unit adds; without P-S the SMS end in a bare colon. Judge it and recommend how integration must handle it.
- The now-dead regex alternatives at mail_club.spec.ts:660.
No builds are needed (JSON only); disk is tight, so create no target/. Do not edit or commit.
End your reply with the `Ready to merge:` line and the report path.
