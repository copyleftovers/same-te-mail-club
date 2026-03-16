# Phase 6: Account Management — Agent Brief

## Read First

1. `spec/Implementation Plan.md` — from "## Phase 6: Account Management" through "## Forbidden Patterns"
2. `spec/User Stories.md` — Story 6.1

## Entry State

Phase 5 complete. Full season lifecycle works: create, launch, enroll, confirm, assign, deliver, receive, complete. SMS batch triggers functional. These files exist on top of previous phases:
- `src/admin/sms.rs` — all 4 SMS batch trigger server functions
- `src/pages/season.rs` — now includes confirm_receipt
- Home page renders all HomeState variants including Assigned and AwaitingReceipt

All E2E tests through Epic 5 pass.

## Scope

This phase is small — one server function addition to an existing file.

`deactivate_participant` in `src/admin/participants.rs`:
1. UPDATE users SET status = 'deactivated'
2. DELETE FROM sessions WHERE user_id = $1 (revoke all active sessions)

## Verification Focus

Deactivation must cascade correctly through ALL existing code. Verify these checks are already in place from previous phases:
- `request_otp` (Phase 2): queries `WHERE status = 'active'` — deactivated user can't get OTP
- `current_user` (Phase 2): checks `status != Active` → returns Unauthorized — existing sessions fail
- `send_season_open_sms` (Phase 5): queries `WHERE status = 'active'` — deactivated user excluded from broadcasts
- Enrollment (Phase 3): should check user status is active before allowing enrollment

If any of these checks are missing from previous phases, ADD them now.

## Leptos Patterns & MCP

Read `ops/leptos-idioms.md` — especially the **MCP Section Index** at the bottom.

**MCP sections to query for this phase** (via `mcp__plugin_leptos-mcp_leptos__get-documentation`):
- `forms-and-actions` — deactivate button is an ActionForm
- `components` — ensure participant list re-renders after deactivation
- `resources` — participant list refetch after mutation

**After writing/modifying any component**: run `leptos-autofixer` on it.

This phase is small but the final gate is FULL regression — if any previous component has latent Leptos issues, now is when they surface. Use the autofixer liberally on any component that fails E2E.

## Traps

- Do NOT soft-delete. Status changes to 'deactivated', user row remains.
- Session deletion is immediate — the `DELETE FROM sessions` ensures existing browser sessions are invalidated on next request.
- The admin participant list should show deactivated users with their status (not hide them).

## E2E Tests

Target: `end2end/tests/mail_club.spec.ts` — `"Epic 6: Account Management"` block AND `"Cancel Season"` block AND `"Season Complete"` block.

This phase runs FULL REGRESSION: all E2E tests must pass. This is the final phase.

## Exit

Run every command in "Phase 6 Verification Gates" from the plan.
The final gate is the FULL E2E suite — all blocks, all stories.
