# Deferred Items

Last updated: 2026-04-28

## Implementation Gaps

### Advance gate checks existence, not release — Story 4.7 AC

Story 4.7 AC says "Assignment → Delivery: advance disabled until assignments are released." Current implementation (`src/admin/season.rs`) computes `assignments_released` as `COUNT(DISTINCT a.sender_id) > 0` — this unblocks when assignments are *generated* (rows exist in DB), not when `release_assignments()` is called. The `release_assignments()` server function only validates, it doesn't write a flag. The gate is weaker than the spec requires. Fix: add a `released_at` timestamp to the season or assignments table, set it when release is called, gate the advance on that.

## Admin UI Redesign

Stories 4.5 (single-page merge) and 4.6 (phase-gated SMS visibility) are written but not implemented. Stories 4.4 (SMS counts), 4.7 (advance gating), and 4.8 (swap dropdowns) are implemented and E2E-tested. The full single-page merge requires user approval of the proposal at `orchestration_log/recon/admin-redesign-proposal.md`.
