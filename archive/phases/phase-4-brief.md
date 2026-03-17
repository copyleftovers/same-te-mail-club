# Phase 4: Assignment — Agent Brief

## Read First

1. `spec/Implementation Plan.md` — from "## Phase 4: Assignment" through "## Phase 5"
2. `spec/Architecture.md` — "Assignment Algorithm" section (algorithm details)
3. `spec/Data Model.md` — assignments, known_groups, known_group_members tables

## Entry State

Phase 3 complete. Seasons, enrollment, and confirm-ready work end-to-end. These files exist on top of previous phases:
- `src/admin/season.rs` — create_season, launch_season, advance_season, cancel_season
- `src/admin/dashboard.rs` — dashboard state server function
- `src/pages/season.rs` — enroll_in_season, confirm_ready, get_season_info
- `src/pages/home.rs` — HomeState enum, get_home_state server function
- Routes: `/`, `/admin`, `/admin/season` all functional

E2E tests for Epic 1, Epic 4, Stories 2.1 and 2.2 pass.

## Key Design Decisions

- `src/assignment.rs` is PURE LOGIC. No `sqlx`, no `PgPool`, no `use_context`, no `leptos`. Only `uuid` and `std` collections. Gate 4 in the plan's verification enforces this with grep.
- No `past_pairings` table. Past pairings are queried from `assignments JOIN seasons WHERE phase IN ('complete', 'cancelled')`. The query lives in `src/admin/assignments.rs`, not the algorithm module.
- No `released` flag anywhere. Release IS the phase transition from assignment → delivery. Gate 5 enforces this.
- `release_assignments` advances the phase to delivery. It does NOT set a per-assignment flag.
- `validate_cycles` must verify: every participant sends to exactly one, receives from exactly one, each cohort forms a single connected loop.

## Leptos Patterns & MCP

Read `ops/leptos-idioms.md` before writing any components — especially the **MCP Section Index** at the bottom.

**MCP sections to query for this phase** (via `mcp__plugin_leptos-mcp_leptos__get-documentation`):
- `forms-and-actions` — admin generate/swap/release forms use ActionForm
- `resources` — cycle visualization loads via Resource + Suspense
- `control-flow` — conditional rendering of cycle visualization, swap UI
- `server-functions` — the admin server fns that bridge algorithm → DB

**After writing each component**: run `leptos-autofixer` on it before moving on.

Key project rules (from idioms file):
- **ActionForm** for all server function forms. `name` attrs must match server fn params.
- **Resource** for data loading. Use `action.version()` as source to refetch after mutations.
- **Tuple syntax** for nested routes: `(StaticSegment("admin"), StaticSegment("assignments"))`.

## Traps

- Cohort splitting: for N <= 15, single cohort. For N > 15, split into groups of 11-15. All groups must be >= 3. The plan says minimize max deviation from mean group size.
- `weight_key(a, b)` uses canonical ordering `(min, max)` — critical for consistent HashMap lookups.
- `generate_cycle`: backtracking DFS with greedy heuristic + multiple random restarts. Keep best (lowest score). This is the most algorithmically complex part — get the unit tests passing first, then optimize.
- Unit tests must be EXTENSIVE: 3 participants, 15 participants, 25 (two cohorts), 30 (two cohorts of 15), social weight influence, invalid topology detection, edge cases N=3, N=11, N=16, N=31.
- `swap_assignment` must validate the resulting topology still forms valid cycles before committing the swap.

## E2E Tests

**Read `end2end/README.md` first** — it is the law for all Playwright code.

Target: `end2end/tests/mail_club.spec.ts` — `"Epic 3: Assignment"` block.
Tests: generate assignments, cycle visualization, swap UI, release, assignment not visible before release.

POM methods you'll exercise:
- `advanceSeason()` — uses `clickAndWaitForResponse()`, no timeout
- `generateAssignments()` — assertion-separated, caller checks `expectCycleVisualization()`
- `releaseAssignments()` — assertion-separated, caller checks for released text

Expected `data-testid` values for assignment components (check POM for full list):
- `generate-button`, `release-button` — admin action buttons
- `cycle-visualization` — the cycle display container
- `recipient-name`, `recipient-phone`, `recipient-branch` — participant assignment view

Every ActionForm button in your Rust components needs the hydration gate pattern and a `data-testid`.

## Exit

Run every command in "Phase 4 Verification Gates" from the plan.
Algorithm purity check: `grep -rn "sqlx\|PgPool\|use_context\|leptos" src/assignment.rs` must return zero matches.
`cargo sqlx prepare --workspace` after all queries.
