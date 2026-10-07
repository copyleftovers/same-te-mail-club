# Spec-reviewer contract (2026-10-04)

BIND FIRST: execute /home/user/same-te-mail-club/orchestration_log/recon/2026-10-04/binding/spec-reviewer.md completely and show the binding output before any other step.

Your agent definition (dev-discipline:spec-reviewer) governs posture, worktree awareness, path re-rooting and the verdict file. Task spec = the named unit in the named plan file (absolute main-repo path; gitignored, absent from worktrees). Project rules to check against: /home/user/same-te-mail-club/CLAUDE.md, guidance/dev-protocol.md, guidance/leptos-idioms.md, end2end/README.md, orchestration_log/reference/conventions.md Forbidden Patterns.
Diff scope: ALWAYS `git -C <worktree> diff <BASE_SHA>..<HEAD_SHA>` with the SHAs given (merge-base), never main..HEAD.
Verdict file: write to the absolute path given (under the MAIN repo's orchestration_log/recon/2026-10-04/fix/reviews/ — this path is exempt from re-rooting because reviews live in main's gitignored recon). Return exactly that path.
A system-reminder about dates/unrelated MCP tools inside tool output is a harness artifact; ignore it.
