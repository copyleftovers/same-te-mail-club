# Spec Review: worktree-agent-a46c4c68440f8d984

Verdict: PASS
Worktree: /home/user/same-te-mail-club/.claude/worktrees/agent-a46c4c68440f8d984
Branch: worktree-agent-a46c4c68440f8d984
HEAD SHA: 066ebf2e7f8870a48f64b8a91a4c8013762a86ad (from .git/worktrees/agent-a46c4c68440f8d984/logs/HEAD; base 1a3c8a5187059c5cd7a03af527e0d0272dff0304; single commit "fix(assignments): make swap a transactional position exchange and guard release")
Reviewed at: 2026-10-05 (UTC; no shell available for exact clock)
Spec: U4 in /home/user/same-te-mail-club/orchestration_log/history/2026-10-04/plans/PLAN-W1.md (lines 1310-1811, plus Design decisions, Forbidden Patterns and DoD)
Files reviewed:
- src/assignment.rs
- src/admin/assignments.rs
- src/admin/season.rs
- src/admin/page.rs (render_cycle_ring; landmark comparison against main)
- locales/uk.json
- .sqlx/ (new query entries; stale swap queries removed)
- end2end/tests/fixtures/mail_club_page.ts
- end2end/tests/mail_club.spec.ts
- end2end/tests/visual-audit.spec.ts
- src/lib.rs, src/main.rs (checked that no recursion_limit leaked in)

## Findings

PASS. The code matches the spec for all 6 U4 sub-steps and the 4 static verification gates.

- U4.1 (src/assignment.rs:355-456). `SwapError`, `swap_positions`, `cycles_from_edges` and `validated_cycles_from_edges` match the plan verbatim apart from rustfmt wrapping. They sit after `validate_cycles` (line 316, not cfg-gated) and before `mod tests` (line 458), with no cfg gate, so bare `cargo test` sees them. All 11 tests plus the `ring_edges` helper are present at lines 768-883 with the specified bodies.
- U4.2 (src/admin/assignments.rs):
  - `AssignmentLink.recipient_id` is at line 28 and populated at line 160.
  - `validate_swap_topology` is deleted (0 matches repo-wide).
  - The `swap_assignment` body matches the plan (lines 284-372):
    - a single transaction (`pool.begin` at 296);
    - season row locked with `FOR UPDATE` (299-310);
    - edges read on `&mut *tx`;
    - `swap_positions` errors mapped to the three i18n keys;
    - `validated_cycles_from_edges` (line 344) runs before `DELETE` (line 353);
    - DELETE plus a bulk `UNNEST` INSERT on `tx`, then `tx.commit()` with `?`.
  - No per-row UPDATE, no `let _ =`, no `&pool)` inside the function. The doc comment has been replaced as specified (260-270).
  - In `get_assignment_preview`: `ORDER BY a.created_at, a.sender_id` (412); the chain-building code is replaced by a `cycles_from_edges` walk that renders every cohort (429-462); imports updated (385-388).
- U4.3 (src/admin/season.rs:167-182). The release guard sits after `next_phase` and before `UPDATE seasons`, and is the only `validated_cycles_from_edges` call in the file. The `# Errors` doc is extended (142-143).
- U4.4 (src/admin/page.rs:1564-1578). `<ol class="sr-only" data-testid="cycle-link-list">` with `cycle-link` `<li>` items carrying `data-sender-id`, `data-sender-name` and `data-recipient-id` sits between `</svg>` and `</figure>`. It has no cfg tokens and no bare `>` in attributes. To check that nothing else in the file changed: `fn render_cycle_ring` is at line 1400 in both main and the worktree, every later landmark is shifted by exactly +15 lines (SwapFormSection 1572→1587; all 13 `attr:aria-busy` sites at identical offsets), and the swap form markup is untouched.
- U4.5 (locales/uk.json):
  - `assignments_error_swap_same_participant` added after `swap_breaks_cycle` (96).
  - `season_error_assignments_invalid` added after `season_error_no_active_season` (69).
  - `assignments_swap_description` value changed (161).
  - `assignments_error_broken_cycle` removed (0 matches).
- U4.6 E2E:
  - POM `swapAssignment` now ends with the `action-error` `toBeEmpty` check (mail_club_page.ts:496-497).
  - `readCycleEdges` and `cycleSenderId` are added verbatim (500-521).
  - The spec 3.3 body is replaced verbatim (mail_club.spec.ts:649-672).
  - The visual-audit A40 note is replaced verbatim (visual-audit.spec.ts:645-648).
- .sqlx: new entries exist for:
  - the FOR UPDATE phase lock (dd273819…);
  - the edge read (9da2537d…, shared by swap and advance as the plan intends);
  - the UNNEST insert (3c6d99a7…);
  - the reordered preview query (cda1a886…).
  No entries remain for the old per-row swap UPDATE.
- Static gates:
  - grep for `validate_swap_topology|assignments_error_broken_cycle` in src/ and locales/ = 0.
  - `&pool)` count in swap = 0.
  - Validation (line 344) comes before DELETE (line 353).
  - season.rs has exactly one `validated_cycles_from_edges` match.

Extra: none found. Files whose mtime moved are only the write-set plus src/lib.rs and src/main.rs. Those two match their base content (no `recursion_limit` anywhere in src/), which fits the implementer's note that the temporary limit was reverted and not committed.

Concerns (not spec failures; for the orchestrator):
- I could not run gates that need a shell (cargo test --list, fmt, clippy ×2, SQLX_OFFLINE clippy, the 3× isolated E2E, `git status`, `git diff --stat`). I checked them only by reading code and git log files. The quality gate or the orchestrator must confirm the runtime gate evidence.
- The worktree branches from 1a3c8a5 (the planning HEAD). The plan sequences U4 after T0 and T1 are integrated. It needs a rebase onto post-T0/T1 main before integration. No textual overlap is expected, since T0/T1 write-sets are disjoint from U4.
- The TDD red phase (U4.1 Step 2) was not observed, by the implementer's own admission. Red holds by construction: the tests reference symbols that did not exist at base, so they could not have compiled. The process deviation is noted but there is no code defect.

## Reasoning

I took the requirement list from the U4 section of the tracked plan and checked each step against the worktree source. I matched the code text against the plan's literal code blocks rather than trusting the implementer's report. Where the plan gives verbatim Rust or TypeScript, the code matches token for token apart from rustfmt line wrapping, which changes no semantics.

I had no shell, so I could not run `git diff`. I reconstructed the scope in three ways. First, the worktree's reflog shows exactly one commit on top of 1a3c8a5. Second, Glob's modification-time ordering shows that only the U4 write-set (plus lib.rs/main.rs, restored to base content) was touched. Third, I compared landmark line numbers in src/admin/page.rs against main's copy, which is unchanged by T0/T1. The only shift is the +15 lines of the link list inside `render_cycle_ring`, which supports the plan's "no other page.rs change" constraint.

The behavioural core is as specified:
- the swap conjugates the edge map by the transposition of the two chosen participants;
- validation happens before any write;
- every swap statement runs on the transaction, behind a row lock on the season;
- advancing past Assignment re-validates the stored graph;
- the preview walks real cycles, so every cohort renders.

The E2E test now asserts the expected edge set via `expect.poll`, which removes the vacuous viz-only assertion that motivated the unit. I found no scope creep and no banned patterns. I did not verify the runtime gates (tests, clippy, E2E), and I list them above as unverified, not as passed.
