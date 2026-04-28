# Deferred Items

Last updated: 2026-04-28

## Missing E2E Test Coverage

### Swap assignment operation (Story 3.3) — HIGH

`swap_assignment` server function (`src/admin/assignments.rs:321`) is the only mutation with zero E2E exercise. Test #38 asserts the override UI element exists (`override-available` testid visible) but never submits a swap.

What's untested:
- The actual swap mutation (DB update of two assignment rows)
- Cycle topology invariant preservation after swap
- Assignment preview refresh after swap
- Error case: invalid swap that would break the cycle

The swap validates Hamiltonian cycle integrity — this is a critical business rule with no E2E coverage. Unit tests for the assignment algorithm exist in `src/assignment.rs` but they don't cover the swap-and-revalidate path.

**To fix:** Add E2E test(s) in the main serial block after test #38. POM needs a `swapAssignment(senderA, senderB)` method. The test should swap two senders, verify the cycle visualization updates, then verify assignments are still releasable.

### Deadline enforcement — MEDIUM

Stories 2.1-AC3 ("enrollment closes at signup deadline") and 2.2-AC2 ("confirm button not shown after deadline") are untestable in the current E2E setup because `SAMETE_TEST_MODE=true` bypasses all deadline checks.

What's untested:
- `enroll_in_season` rejects enrollment after signup deadline
- `confirm_ready` rejects confirmation after confirm deadline
- UI hides enroll/confirm buttons after their respective deadlines

**To fix:** Either add a dedicated test that creates a season with past deadlines (requires `SAMETE_TEST_MODE=false` for that test run, which may need a separate server instance), or add unit tests for the deadline-gating logic in the server functions.

### 404 fallback route — LOW

No test navigates to a non-existent path to exercise the fallback handler (`src/app.rs:99`). Cosmetic — no product story, no user impact.

**To fix:** Single test: `page.goto("/nonexistent")`, assert page contains not-found text.

## Missing User Stories

These features are implemented and E2E-tested but have no formal user story with acceptance criteria in `spec/technical/User Stories.md`. Documenting the gap here; story authoring is a separate task.

| Feature | Server function | E2E coverage | Notes |
|---------|----------------|-------------|-------|
| Season cancellation | `cancel_season` | Tests 51-53 | No Story 4.3. Cancellation sets phase to `cancelled` from any non-terminal phase. |
| Logout | `logout` | Tests 57-58 | Story 1.2 covers auth but never specifies logout as a user action. |
| Admin dashboard | `get_dashboard` | Tests 11, 15, 23, 32, 41, 48, 49, 52 | Was a deferred item, now implemented. No story defines what the dashboard shows or its acceptance criteria. |

## Implicit-Only Coverage

These acceptance criteria pass only because page navigation triggers their server functions. No test directly validates the behavior.

| AC | What it specifies | How it's "covered" | Risk |
|----|-------------------|-------------------|------|
| 2.1-AC6 | No withdrawal mechanism | No test attempts to un-enroll | Low — no un-enroll endpoint exists |
| 2.2-AC4 | Confirmation is irreversible | Test 29 checks button absence, but doesn't attempt a direct `confirm_ready` call after already confirmed | Low — server function is idempotent (`ON CONFLICT DO NOTHING` equivalent via `confirmed_ready_at` latch) |
| 2.3-AC4 | Participant sees only their own recipient | Test 43 counts exactly one `recipient-name` element | Adequate |
| 2.3-AC5 | No info about who sends to the participant | No test verifies absence of sender info | Low — the `get_home_state` query simply doesn't fetch it |
