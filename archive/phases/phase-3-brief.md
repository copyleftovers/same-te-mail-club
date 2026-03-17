# Phase 3: Season Lifecycle — Agent Brief

## Read First

1. `spec/Implementation Plan.md` — from "## Phase 3: Season Lifecycle" through "## Phase 4"
2. `spec/Data Model.md` — seasons, enrollments tables
3. `spec/User Stories.md` — Stories 4.1, 4.2, 2.1, 2.2

## Entry State

Phase 2 complete. Auth works end-to-end. These files exist on top of Phase 1:
- `src/sms.rs` — SMS client with dry-run support (SSR-only)
- `src/auth.rs` — OTP, sessions, current_user helper (SSR-only)
- `src/pages/login.rs` — login page with server functions
- `src/pages/onboarding.rs` — onboarding page with server function
- `src/admin/participants.rs` — participant registration + listing
- `src/app.rs` — routes for `/login`, `/onboarding`, `/`, `/admin/participants`
- `src/pages.rs`, `src/admin/mod.rs` — module declarations

E2E Epic 1 tests pass. Users can register, login, and onboard.

## Correction: Season Create vs Launch

The Implementation Plan says "Creation itself IS the launch." This conflicts with the E2E tests, which require TWO separate steps:

1. `createSeason()` — admin fills form, clicks create → season exists but NOT visible to participants
2. `launchSeason()` — admin clicks launch button → enrollment opens to participants

The `seasons` table has a `launched_at TIMESTAMPTZ` column (added in Phase 1). Implementation:

- `create_season` server function: INSERT with `launched_at = NULL`. Season exists in DB with phase = 'enrollment' but is not visible to participants.
- **NEW** `launch_season` server function: `UPDATE seasons SET launched_at = now() WHERE id = $1 AND launched_at IS NULL`. Requires admin. Returns error if already launched or no active season.
- `advance_season`: must check `launched_at IS NOT NULL` before allowing any phase advance. Cannot advance an unlaunched season.
- All participant-facing queries for active seasons: add `AND launched_at IS NOT NULL` to the WHERE clause.
- Dashboard: show "created/створено" when `launched_at IS NULL`, show phase name when launched.

The E2E tests expect:
- After create: dashboard shows `/created|створено/i`
- Before launch: participant does NOT see enroll button
- After launch: dashboard shows `/enrollment|signup|реєстрація/i`, participant sees enroll button

The POM has `launchSeason()` which navigates to `/admin/season` and clicks `[data-testid="launch-button"]`.

## Other Key Points

- Enrollment does NOT store Nova Poshta data. No NP fields on enrollments table.
- `confirm_ready` uses `confirmed_ready_at` timestamp (nullable), NOT a boolean.
- `HomeState` enum is the centerpiece of the home page — one match expression, no scattered conditionals.
- The `Assigned` variant's NP data comes from `delivery_addresses` JOIN, not enrollments.

## Leptos Patterns & MCP

Read `ops/leptos-idioms.md` before writing any components — especially the **MCP Section Index** at the bottom.

**MCP sections to query for this phase** (via `mcp__plugin_leptos-mcp_leptos__get-documentation`):
- `mental-model` — **read FIRST** before writing any component
- `forms-and-actions` — season creation, enrollment, confirm-ready all use ActionForm
- `server-functions` — all server fns in this phase (create/launch/advance/cancel season, enroll, confirm)
- `resources` — HomeState, DashboardState load via Resource + Suspense
- `control-flow` — HomeState enum match rendering (conditional views per variant)
- `routing` — new admin routes use tuple syntax

**After writing each component**: run `leptos-autofixer` on it before moving on.

Key project rules (from idioms file):
- **ActionForm** for all server function forms. `name` attrs must match server fn params. No `on:input` → signal → dispatch.
- **Resource** for data loading. Use `action.version()` as source to refetch after mutations.
- **Tuple syntax** for nested routes: `(StaticSegment("admin"), StaticSegment("season"))`.
- **`expect_context::<T>()`** for server-side context (pool, config).

## Traps

- The plan's `advanceSeason` references E2E test names that used the old 8-phase model. The E2E tests have been updated to match the 6-phase model. Phase transitions are: enrollment → preparation → assignment → delivery → complete.
- `enroll_in_season` must verify the user has a delivery address (row in `delivery_addresses`).
- Season creation: the `one_active_season` unique index prevents two active seasons. Handle the unique violation gracefully.
- The plan mentions `advance_season` but doesn't mention `launch_season` — you need to add `launch_season` as described in the Correction section above.

## E2E Tests

**Read `end2end/README.md` first** — it is the law for all Playwright code.

Target: `end2end/tests/mail_club.spec.ts`
- `"Epic 4: Season Management"` block (stories 4.1, 4.2)
- `"Story 2.1: Enrollment"` block
- `"Story 2.2: Confirm Ready"` block

POM methods you'll exercise (already implemented in `end2end/tests/fixtures/mail_club_page.ts`):
- `createSeason()` — self-contained, waits for launch-button to appear
- `launchSeason()` — self-contained, waits for advance-button to appear
- `advanceSeason()` — uses `clickAndWaitForResponse()`, no timeout
- `cancelSeason()` — self-contained, waits for cancel-button to disappear
- `enrollInSeason()` — assertion-separated, caller checks `expectEnrolled()`
- `confirmReady()` — assertion-separated, caller checks `expectConfirmed()`

Every Rust component with an ActionForm MUST have the hydration gate:
```rust
let (hydrated, set_hydrated) = signal(false);
Effect::new(move |_| set_hydrated.set(true));
// button: disabled=move || !hydrated.get()
```

Every actionable element MUST have `data-testid`. Check the POM for expected testid values.

## Exit

Run every command in "Phase 3 Verification Gates" from the plan.
`cargo sqlx prepare --workspace` after all queries.
