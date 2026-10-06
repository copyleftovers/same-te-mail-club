RESUME (killed by a session limit; the machine was also restored from an older disk snapshot).
- Your worktree was rebuilt from the branch snapshot of 2026-10-05 21:15 UTC (orchestration_log/history/2026-10-04/wip/): your commits were re-applied (new SHAs) and your uncommitted diff re-applied on top. Anything you did after 21:15 UTC is lost — check `git log` + `git status` against your memory and redo only what is missing.
- target/ is empty: rebuild with CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0. Postgres sibling DBs may be gone: recreate as your plan says.
- COMMIT EARLY AND OFTEN (each green sub-step), one-line conventional messages. Never stop idle.
- If your unit touches swap/assignment code or POM swap helpers: `git merge --no-edit claude/loving-johnson-7l8hn5` (U4 merged there).
- Parallelism directive: /home/user/same-te-mail-club/orchestration_log/recon/2026-10-04/fix/prompts/_parallelism.md.
Report once at the end in the implementer Report Format.
