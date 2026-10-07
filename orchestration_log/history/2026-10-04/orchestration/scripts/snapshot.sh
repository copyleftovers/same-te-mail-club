#!/bin/bash
# Every 10 min: snapshot each lane worktree's committed+uncommitted work as patches on the integration branch, so nothing lives only on this machine.
R=/home/user/same-te-mail-club; W=orchestration_log/history/2026-10-04/wip
while true; do
  cd $R || exit 1
  BASE=$(git rev-parse claude/loving-johnson-7l8hn5)
  : > $W/INDEX.txt
  for wt in .claude/worktrees/agent-*; do
    id=${wt##*/}; git -C $wt rev-parse HEAD >/dev/null 2>&1 || continue
    git -C $wt add -A -N 2>/dev/null
    mb=$(git -C $wt merge-base HEAD $BASE)
    git -C $wt format-patch --stdout $mb..HEAD > $W/$id.commits.patch
    git -C $wt diff HEAD > $W/$id.uncommitted.patch
    echo "$id $(git -C $wt branch --show-current) head=$(git -C $wt rev-parse --short HEAD) base=$mb commits=$(git -C $wt rev-list --count $mb..HEAD)" >> $W/INDEX.txt
  done
  find $W -size 0 -delete
  for i in 1 2 3; do git add $W && git commit -q -m "docs(campaign): snapshot in-flight lane work" -- $W >/dev/null 2>&1; git push -q origin claude/loving-johnson-7l8hn5 >/dev/null 2>&1 && break; sleep 20; done
  sleep 600
done
