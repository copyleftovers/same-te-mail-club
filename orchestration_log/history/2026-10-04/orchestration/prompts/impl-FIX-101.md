Read /home/user/same-te-mail-club/orchestration_log/recon/2026-10-04/fix/prompts/_implementer.md and obey it (binding, read-path exemption, build env). Also obey /home/user/same-te-mail-club/orchestration_log/recon/2026-10-04/fix/prompts/_parallelism.md.
Unit FIX-101: fix problem PRB-101 in /home/user/same-te-mail-club/orchestration_log/history/2026-10-04/PROBLEMS.md. Scope: the "F-COV addendum (2026-10-07, PRB-101)" in /home/user/same-te-mail-club/orchestration_log/history/2026-10-04/plans/PLAN-findings.md. Nothing else from F-COV.
Base: first `git merge --no-edit claude/loving-johnson-7l8hn5` (or branch from it).
Root cause (verified by the T1 and P-S reviews): `deactivateParticipant` in end2end/tests/fixtures/mail_club_page.ts waits on the page-wide `inactive-status` testid. With an earlier deactivated participant, that wait either passes vacuously or fails strict mode (2 matches), e.g. visual-audit.spec.ts:798 and mail_club.spec.ts ~881.
Do:
1. Scope that wait, and the spec check, to the clicked participant's row (row testid plus name filter), so it waits for THIS row's status to change.
2. Grep the POM and specs for every other page-wide status/badge testid used as a completion wait after a per-row action, and scope each one.
3. Add any missing row-identifying testid in Rust only if needed, keeping markup otherwise identical.
4. No waitForTimeout, no networkidle, testids only (end2end/README.md).
Gates, in the FOREGROUND:
- fmt, both clippies and cargo test, if Rust was touched;
- `isolated-capture.sh e2e_fix101 full` with PLAYWRIGHT_BROWSERS_PATH=/opt/pw-browsers: 3 CONSECUTIVE green runs; paste the summary lines.
- RED proof: with only step 1 reverted, show the strict-mode failure reproduces, or explain deterministically why the old wait was vacuous.
Never `pkill -f cargo`. Commit with one-line conventional messages and report in the implementer Report Format.
