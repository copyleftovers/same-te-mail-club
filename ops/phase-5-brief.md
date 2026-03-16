# Phase 5: Delivery + SMS — Agent Brief

## Read First

1. `spec/Implementation Plan.md` — from "## Phase 5: Delivery + SMS" through "## Phase 6"
2. `spec/User Stories.md` — Stories 2.3, 2.4, 5.1, 5.2, 5.3, 5.4

## Entry State

Phase 4 complete. Assignment algorithm works, admin can generate/swap/release assignments. These files exist on top of previous phases:
- `src/assignment.rs` — pure algorithm module (cohort split, cycle generation, validation, weight matrix)
- `src/admin/assignments.rs` — generate, swap, release server functions

E2E tests for Epics 1, 3, 4, and Stories 2.1, 2.2 pass.

## Key Design Decisions

- Assignment visibility is gated by `season phase = Delivery`. No `released` flag. No per-assignment visibility check. The `get_home_state` function (Phase 3) already has the `Assigned` variant — it just needs the query to populate it when phase = delivery.
- Nova Poshta data comes from `delivery_addresses` JOIN, NOT from enrollments. Query pattern:
  ```sql
  SELECT u.name, u.phone, da.nova_poshta_city, da.nova_poshta_number
  FROM assignments a
  JOIN users u ON u.id = a.recipient_id
  JOIN delivery_addresses da ON da.user_id = a.recipient_id
  WHERE a.sender_id = $1 AND a.season_id = $2
  ```
- Receipt is on the `assignments` row (`receipt_status` enum column). No separate `receipts` table. No JOINs to a receipts table.
- `confirm_receipt`: the participant is the RECIPIENT (not the sender). Query by `recipient_id`.
- `notified_at` tracks per-assignment SMS delivery. `send_assignment_sms` sets it on success, leaves NULL on failure. Organizer sees who wasn't reached.
- All SMS sends go through `sms::send_sms` (from Phase 2). No direct TurboSMS API calls outside `sms.rs`.
- All user-facing SMS text is in Ukrainian.

## Leptos Patterns & MCP

Read `ops/leptos-idioms.md` before writing any components — especially the **MCP Section Index** at the bottom.

**MCP sections to query for this phase** (via `mcp__plugin_leptos-mcp_leptos__get-documentation`):
- `forms-and-actions` — receipt confirmation form, SMS trigger buttons all use ActionForm
- `server-functions` — SMS batch triggers, receipt confirmation server fns
- `control-flow` — HomeState match rendering for Assigned/AwaitingReceipt variants
- `resources` — SMS report display loads asynchronously
- `error-handling` — SMS delivery failures need proper error surface

**After writing each component**: run `leptos-autofixer` on it before moving on.

Key project rules (from idioms file):
- **ActionForm** for all server function forms. `name` attrs must match server fn params.
- **Resource** for data loading. Use `action.version()` as source to refetch after mutations.

## Traps

- The home page already has `HomeState::Assigned` and `HomeState::AwaitingReceipt` variants from Phase 3. You're completing the `get_home_state` logic to populate these during delivery phase. Don't create new page components — extend the existing home page match.
- Receipt confirmation: `receipt_status` starts at `NoResponse`. Once set to `Received` or `NotReceived`, it cannot be changed (one-way). Check `receipt_status = 'no_response'` before updating.
- `send_season_open_sms` targets ALL active users, not just enrolled ones (it's a "season is open" broadcast).
- `send_assignment_sms` targets senders with `notified_at IS NULL` (haven't been notified yet).
- `send_confirm_nudge_sms` targets enrolled users with `confirmed_ready_at IS NULL` (haven't confirmed).
- `send_receipt_nudge_sms` targets recipients with `receipt_status = 'no_response'`.

## E2E Tests

Target: `end2end/tests/mail_club.spec.ts`
- `"Stories 2.3–2.4: Delivery & Receipt"` block
- SMS trigger tests are interspersed: story 5.3 is in Epic 4 block, stories 5.1/5.2/5.4 are in their respective blocks.

## Exit

Run every command in "Phase 5 Verification Gates" from the plan.
`cargo sqlx prepare --workspace` after all queries.
