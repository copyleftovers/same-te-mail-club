# Deferred Items

Last updated: 2026-04-28

## Implementation Gaps (Stories written, code doesn't match yet)

### Cancel confirmation dialog — Story 4.3 AC

Story 4.3 specifies "cancellation requires a confirmation step before submitting." Current implementation (`src/admin/season.rs:462-477`) submits immediately via ActionForm with no confirmation. Needs a confirmation dialog or two-step flow.

### Distinct cancelled-season participant UX — Story 4.3 AC

Story 4.3 specifies "participants see a distinct 'season cancelled' state." Current implementation (`src/pages/home.rs`) shows the generic "no active season" message for both cancelled seasons and no-season states. Needs a new `HomeState` variant for cancelled seasons.

### Envelope reveal re-implementation — Story 2.3 AC

The envelope reveal concept is a confirmed product decision (Story 2.3 AC describes the intended behavior). Current implementation (`src/pages/home.rs:768-903`) is non-functional and needs thorough investigation and re-implementation. Known issues include: localStorage key not season-scoped (should include season ID per Story 2.3 AC "reveal state persisted per season").

## Admin UI Redesign

The admin interface (5 pages: dashboard, season, participants, assignments, sms) separates data from the actions it informs. For a ~15-participant single-page app, this creates unnecessary navigation overhead. Needs a full deconstruct-reconstruct cycle to co-locate relevant data with its related actions. Story 4.4 documents the current dashboard as interim.

## Missing E2E Test Coverage

### Swap assignment operation (Story 3.3) — HIGH

`swap_assignment` server function (`src/admin/assignments.rs:321`) is the only mutation with zero E2E exercise. Test #38 asserts the override UI element exists (`override-available` testid visible) but never submits a swap.

What's untested:
- The actual swap mutation (DB update of two assignment rows)
- Cycle topology invariant preservation after swap
- Assignment preview refresh after swap
- Error case: invalid swap that would break the cycle

**To fix:** Add E2E test(s) in the main serial block after test #38. POM needs a `swapAssignment(senderA, senderB)` method. The test should swap two senders, verify the cycle visualization updates, then verify assignments are still releasable.

### Deadline enforcement — MEDIUM

Stories 2.1-AC3 ("enrollment closes at signup deadline") and 2.2-AC2 ("confirm button not shown after deadline") are untestable in the current E2E setup because `SAMETE_TEST_MODE=true` bypasses all deadline checks.

What's untested:
- `enroll_in_season` rejects enrollment after signup deadline
- `confirm_ready` rejects confirmation after confirm deadline
- UI hides enroll/confirm buttons after their respective deadlines

**To fix:** Either add a dedicated test that creates a season with past deadlines (requires `SAMETE_TEST_MODE=false` for that test run, which may need a separate server instance), or add unit tests for the deadline-gating logic in the server functions.

### 404 fallback route — LOW

No test navigates to a non-existent path to exercise the fallback handler (`src/app.rs:99`). No product story, no user impact.

**To fix:** Single test: `page.goto("/nonexistent")`, assert page contains not-found text.

## Implicit-Only Coverage

These acceptance criteria pass only because page navigation triggers their server functions. No test directly validates the behavior.

| AC | What it specifies | How it's "covered" | Risk |
|----|-------------------|-------------------|------|
| 2.1-AC6 | No withdrawal mechanism | No test attempts to un-enroll | Low — no un-enroll endpoint exists |
| 2.2-AC4 | Confirmation is irreversible | Test 29 checks button absence, but doesn't attempt a direct `confirm_ready` call after already confirmed | Low — server function is idempotent via `confirmed_ready_at` latch |
| 2.3-AC4 | Participant sees only their own recipient | Test 43 counts exactly one `recipient-name` element | Adequate |
| 2.3-AC5 | No info about who sends to the participant | No test verifies absence of sender info | Low — the `get_home_state` query simply doesn't fetch it |
