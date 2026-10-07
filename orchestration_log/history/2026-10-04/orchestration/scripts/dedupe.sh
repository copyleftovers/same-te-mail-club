#!/bin/bash
# Periodically hardlink identical cargo dep artifacts across lane worktrees to fit parallel builds in the disk allowance.
while true; do
  cd /home/user/same-te-mail-club/.claude/worktrees 2>/dev/null && timeout 300 hardlink -c -q -m 65536 */target/*/deps */target/*/*/deps 2>/dev/null
  sleep 180
done
