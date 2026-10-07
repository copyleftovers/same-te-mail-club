Background loops for the campaign (start with run_in_background):
- snapshot.sh — every 10 min, writes wip/<worktree>.{commits,uncommitted}.patch + INDEX.txt (base = full merge-base sha) and pushes. Restore a worktree: checkout --detach <base>; git am --3way <id>.commits.patch; git apply --3way <id>.uncommitted.patch; git branch -f <branch> HEAD.
- dedupe.sh — every 2 min, hardlinks identical cargo dep artifacts across worktree target/ dirs (disk allowance).
