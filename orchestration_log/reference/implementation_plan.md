# Implementation Plan: Close All Deferred Items

Last updated: 2026-04-28

## Preamble

Seven items remain open before the project backlog is clean. Three are implementation gaps (stories written, code doesn't match), three are missing E2E tests, and one is a design analysis. This plan is executable cold — no conversation context required.

Reference: `orchestration_log/reference/deferred_items.md` defines WHAT is open. This file defines HOW to close each item.

Skill prerequisites: `dev-discipline:defensive-planning`, `dev-discipline:tdd`, `dev-discipline:triage-issue`, `dev-discipline:improve-architecture`, `frontend-design:frontend-design`, `orchestration:agentic-delegation`, `dev-discipline:dev-orchestration`.

Agent types: `dev-discipline:implementer`, `dev-discipline:spec-reviewer`, `dev-discipline:code-quality-reviewer`, `qa-automation:executor-agent`, `qa-automation:healer-agent`.

---

## Phase 1: Implementation Fixes (Parallel)

### Task 1A: Cancel Confirmation Dialog

**Story:** 4.3 AC — "Cancellation requires a confirmation step before submitting."

**What exists now:**
- `src/admin/season.rs:460-481` — `ActionForm` wraps a single button (`data-testid="cancel-button"`, `data-variant="destructive"`). Clicking submits immediately.
- POM `cancelSeason()` at `end2end/tests/fixtures/mail_club_page.ts:329-337` clicks `cancel-button` and waits for it to disappear.
- E2E test 52 (`end2end/tests/mail_club.spec.ts:561-567`) calls `cancelSeason()`.

**What to build:**
Replace the single-click ActionForm with a two-step flow: first click shows a confirmation prompt, second click submits.

**Implementation (Leptos pattern):**
Add a `confirming` signal. The cancel button toggles `confirming` to `true`. When `confirming` is `true`, render a confirmation panel with two buttons: "Confirm cancel" (submits the ActionForm) and "Back" (resets `confirming` to `false`).

**File:** `src/admin/season.rs`

**Step 1:** Add signal inside `ActiveSeasonPanel` (after line 366):
```rust
let (confirming, set_confirming) = signal(false);
```

**Step 2:** Replace the cancel button block (lines 460-481) with:
```rust
{if launched {
    view! {
        <div data-testid="cancel-section">
            <Show when=move || !confirming.get()>
                <button
                    class="btn"
                    data-variant="destructive"
                    type="button"
                    data-testid="cancel-button"
                    disabled=move || !hydrated.get()
                    on:click=move |_| set_confirming.set(true)
                >
                    {t!(i18n, season_cancel_button)}
                </button>
            </Show>
            <Show when=move || confirming.get()>
                <div data-testid="cancel-confirmation">
                    <p>{t!(i18n, season_cancel_confirm_prompt)}</p>
                    <div class="flex gap-3">
                        <leptos::form::ActionForm action=cancel_action>
                            <button
                                class="btn"
                                data-variant="destructive"
                                type="submit"
                                data-testid="cancel-confirm-button"
                                disabled=move || cancel_pending.get()
                            >
                                {move || if cancel_pending.get() {
                                    "...".into_any()
                                } else {
                                    t!(i18n, season_cancel_confirm_yes).into_any()
                                }}
                            </button>
                        </leptos::form::ActionForm>
                        <button
                            class="btn"
                            data-variant="secondary"
                            type="button"
                            data-testid="cancel-back-button"
                            on:click=move |_| set_confirming.set(false)
                        >
                            {t!(i18n, season_cancel_confirm_no)}
                        </button>
                    </div>
                </div>
            </Show>
        </div>
    }
    .into_any()
} else {
    ().into_any()
}}
```

**Step 3:** Add i18n keys to `locales/uk.json`:
```json
"season_cancel_confirm_prompt": "Ви впевнені? Це вплине на всіх зареєстрованих учасників.",
"season_cancel_confirm_yes": "Так, скасувати",
"season_cancel_confirm_no": "Назад"
```

**Step 4:** Update POM `cancelSeason()` at `end2end/tests/fixtures/mail_club_page.ts:329-337`:
```typescript
async cancelSeason() {
  await this.page.goto("/admin/season");
  // Step 1: Click cancel to show confirmation
  await this.page.getByTestId("cancel-button").click();
  await expect(this.page.getByTestId("cancel-confirmation")).toBeVisible();
  // Step 2: Confirm the cancellation
  await this.clickAndWaitForResponse(
    this.page.getByTestId("cancel-confirm-button"),
    "cancel_season",
  );
  await expect(this.page.getByTestId("cancel-confirm-button")).not.toBeVisible();
}
```

**Step 5:** Reset `confirming` on successful cancel. After the toast Effect (line 536-540 in season.rs), add:
```rust
Effect::new(move |_| {
    if let Some(Ok(())) = cancel_action.value().get() {
        set_confirming.set(false);
    }
});
```

**Verification gates:**

```bash
# Gate 1: Compiles clean
cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tail -5
# REQUIRED OUTPUT: no warnings/errors

# Gate 2: i18n keys exist
grep -c "season_cancel_confirm" locales/uk.json
# REQUIRED OUTPUT: 3

# Gate 3: TestIDs exist in source
grep -c 'cancel-confirmation\|cancel-confirm-button\|cancel-back-button' src/admin/season.rs
# REQUIRED OUTPUT: 3

# Gate 4: POM uses two-step flow
grep -c 'cancel-confirmation\|cancel-confirm-button' end2end/tests/fixtures/mail_club_page.ts
# REQUIRED OUTPUT: 2
```

---

### Task 1B: Distinct Cancelled-Season Participant UX

**Story:** 4.3 AC — "After cancellation, participants see a distinct 'season cancelled' state."

**What exists now:**
- `HomeState` enum at `src/pages/home.rs:14-54` has no `Cancelled` variant.
- `get_home_state()` at line 249-268 maps cancelled seasons to `HomeState::NoSeason`.
- E2E test 53 (`end2end/tests/mail_club.spec.ts:570-575`) asserts "no season" text.

**What to build:**
Add a `Cancelled` variant to `HomeState`. Update `get_home_state()` to return it. Add a view branch that shows "season was cancelled" messaging. Update E2E test 53.

**File:** `src/pages/home.rs`

**Step 1:** Add variant to `HomeState` enum (after `Complete` at line 53):
```rust
/// Season was cancelled by the organizer.
Cancelled,
```

**Step 2:** Update `get_home_state()` (lines 258-267). Change:
```rust
return match most_recent_phase {
    Some(Phase::Complete) => Ok(HomeState::Complete),
    _ => Ok(HomeState::NoSeason),
};
```
To:
```rust
return match most_recent_phase {
    Some(Phase::Complete) => Ok(HomeState::Complete),
    Some(Phase::Cancelled) => Ok(HomeState::Cancelled),
    _ => Ok(HomeState::NoSeason),
};
```

**Step 3:** Add view branch in the main `match` (in `render_home_content`, find the `HomeState::Complete` arm and add after it):
```rust
HomeState::Cancelled => view! {
    <div data-testid="season-cancelled">
        <h2 class="font-display text-xl">{t!(i18n, home_season_cancelled_title)}</h2>
        <p class="text-muted">{t!(i18n, home_season_cancelled_body)}</p>
    </div>
}
.into_any(),
```

**Step 4:** Add i18n keys to `locales/uk.json`:
```json
"home_season_cancelled_title": "Сезон скасовано",
"home_season_cancelled_body": "Організатор скасував поточний сезон. Слідкуйте за SMS — ми повідомимо, коли почнеться новий."
```

**Step 5:** Run `cargo sqlx prepare --workspace` (the query in `get_home_state` doesn't change SQL, but verify).

**Step 6:** Update E2E test 53 (`end2end/tests/mail_club.spec.ts:570-575`). Change assertion from:
```typescript
await app.expectHomeContent(/no season|немає сезону|SMS/i);
```
To:
```typescript
await expect(app.page.getByTestId("season-cancelled")).toBeVisible();
```

**Verification gates:**

```bash
# Gate 1: Compiles clean
cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tail -5
# REQUIRED OUTPUT: no warnings/errors

# Gate 2: HomeState::Cancelled exists
grep -c "Cancelled" src/pages/home.rs | head -1
# REQUIRED OUTPUT: at least 3 (enum, match arm in get_home_state, match arm in render)

# Gate 3: i18n keys exist
grep -c "home_season_cancelled" locales/uk.json
# REQUIRED OUTPUT: 2

# Gate 4: TestID in source
grep -c 'season-cancelled' src/pages/home.rs
# REQUIRED OUTPUT: 1

# Gate 5: E2E test uses testid
grep -c 'season-cancelled' end2end/tests/mail_club.spec.ts
# REQUIRED OUTPUT: 1
```

---

## Phase 2: E2E Test Additions

### Task 2A: Swap Assignment E2E Test

**Story:** 3.3 — only untested mutation. HIGH priority.

**What exists now:**
- Swap form at `src/admin/assignments.rs:960-1009` takes raw UUID strings in text inputs (`sender-a`, `sender-b`).
- Submit button: `data-testid="swap-button"`.
- Section wrapper: `data-testid="override-available"`.
- Cycle visualization nodes: `data-testid="node-{sanitized-name}"` (name lowercased, spaces→dashes).
- Test #38 only asserts `override-available` is visible.
- No POM method for swap exists.
- The swap form requires user UUIDs, but the cycle visualization only shows names.

**Pre-requisite source change:** Add `data-user-id` attribute to cycle visualization nodes so the E2E test can read sender UUIDs from the DOM.

**File:** `src/admin/assignments.rs`

**Step 1:** Add `sender_id` field to `AssignmentLink` struct (line 28):
```rust
pub struct AssignmentLink {
    pub sender_id: String,
    pub sender_name: String,
    pub recipient_name: String,
}
```

**Step 2:** Populate `sender_id` in `get_assignment_preview()` — update the query and mapping (around lines 520-540) to include the sender's user ID.

**Step 3:** Add `data-user-id` attribute to the cycle node element in `render_cycle_ring()` (around line 904):
```rust
data-user-id=link.sender_id.clone()
```

**Step 4:** Run `cargo sqlx prepare --workspace`.

**Step 5:** Add POM method to `end2end/tests/fixtures/mail_club_page.ts`:
```typescript
async swapAssignment(senderNameA: string, senderNameB: string) {
  await this.page.goto("/admin/assignments");
  // Read UUIDs from cycle visualization nodes
  const nodeA = this.page.getByTestId(`node-${senderNameA.toLowerCase().replace(/ /g, "-")}`);
  const nodeB = this.page.getByTestId(`node-${senderNameB.toLowerCase().replace(/ /g, "-")}`);
  const idA = await nodeA.getAttribute("data-user-id");
  const idB = await nodeB.getAttribute("data-user-id");
  if (!idA || !idB) throw new Error("Could not read user IDs from cycle nodes");
  // Fill swap form
  await expect(this.page.getByTestId("swap-button")).toBeEnabled();
  await this.page.locator("#sender-a").fill(idA);
  await this.page.locator("#sender-b").fill(idB);
  // Submit and wait for preview refresh
  await this.clickAndWaitForResponse(
    this.page.getByTestId("swap-button"),
    "swap_assignment",
  );
  await expect(this.page.getByTestId("cycle-visualization")).toBeVisible();
}
```

**Step 6:** Add test after test #38 in `end2end/tests/mail_club.spec.ts`. Insert inside the Epic 3 block, before the release test:
```typescript
test("3.3 — admin swaps two assignments", async ({ page }) => {
  const app = new MailClubPage(page);
  await app.login(ADMIN_PHONE);
  // Get two participant names from the registered test data
  await app.swapAssignment(NAMES.A, NAMES.B);
  // Verify cycle visualization still exists (topology preserved)
  await app.expectCycleVisualization();
});
```

Note: `NAMES.A` and `NAMES.B` must match the names used in `registerParticipant()` calls at the top of the test suite. Check the test file for the exact name constants.

**Verification gates:**

```bash
# Gate 1: Compiles clean
cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tail -5
# REQUIRED OUTPUT: no warnings/errors

# Gate 2: data-user-id in source
grep -c 'data-user-id' src/admin/assignments.rs
# REQUIRED OUTPUT: at least 1

# Gate 3: POM method exists
grep -c 'swapAssignment' end2end/tests/fixtures/mail_club_page.ts
# REQUIRED OUTPUT: at least 1

# Gate 4: Test exists
grep -c 'admin swaps two assignments' end2end/tests/mail_club.spec.ts
# REQUIRED OUTPUT: 1
```

---

### Task 2B: 404 Fallback E2E Test

**What exists now:**
- `src/app.rs:99`: `<Routes fallback=move || t!(i18n, app_not_found)>` — renders i18n text, no testid.
- No E2E test for non-existent routes.

**Pre-requisite source change:** Wrap the fallback text in an element with a testid.

**File:** `src/app.rs`

**Step 1:** Change the fallback from raw text to a wrapped element:
```rust
<Routes fallback=move || view! {
    <p data-testid="not-found">{t!(i18n, app_not_found)}</p>
}>
```

**Step 2:** Add test in `end2end/tests/mail_club.spec.ts`. Add as a standalone test at the end of the main serial block (before independent blocks):
```typescript
test("404 — non-existent route shows not-found", async ({ page }) => {
  await page.goto("/this-route-does-not-exist");
  await expect(page.getByTestId("not-found")).toBeVisible();
});
```

**Verification gates:**

```bash
# Gate 1: TestID in source
grep -c 'not-found' src/app.rs
# REQUIRED OUTPUT: at least 1

# Gate 2: Test exists
grep -c 'non-existent route' end2end/tests/mail_club.spec.ts
# REQUIRED OUTPUT: 1
```

---

## Phase 3: Envelope Reveal Investigation

**Story:** 2.3 AC — "horrible non-working implementation" (user's words).

**Use the `dev-discipline:triage-issue` skill.** Do NOT skip straight to implementation.

**Known issues from code review:**
1. localStorage key `"assignment_revealed"` is global — not season-scoped (line 784, 818 of `home.rs`). Cross-season pollution: revealing in season 1 auto-reveals in season 2.
2. `setup_envelope_reveal()` creates fresh signals on every render call (line 805). No cleanup.
3. No test for cross-season persistence behavior.

**Triage steps:**
1. Start the dev server (`just dev` in background).
2. Use `qa-automation:planner-agent` to explore the live app at the Assigned state. Document what actually happens when clicking the envelope.
3. Test cross-session behavior: reveal, reload, verify persistence. Clear localStorage, reload, verify reset.
4. Document all broken behaviors in `orchestration_log/recon/envelope-triage.md`.

**After triage:** Write a correction plan for the envelope using `defensive-planning`. The fix must:
- Scope localStorage key to season ID: `assignment_revealed_{season_uuid}`
- Ensure `setup_envelope_reveal()` receives the season ID as a parameter
- Add E2E test verifying reveal state does not leak across seasons (may require creating two seasons sequentially in the test)

**Do NOT implement before the triage report is reviewed by the orchestrator.**

---

## Phase 4: Deadline Enforcement Testing

**Stories:** 2.1-AC3, 2.2-AC2.

**What exists now:**
- `enroll_in_season` at `src/pages/home.rs:314-318`: checks `SAMETE_TEST_MODE` env var. If `true`, skips deadline comparison entirely.
- `confirm_ready` at `src/pages/home.rs:414-418`: identical bypass.
- Both use: `std::env::var("SAMETE_TEST_MODE").as_deref() == Ok("true")`.
- Deadline comparison: `season.signup_deadline < time::OffsetDateTime::now_utc()`.

**Testing approach:** Unit tests for the deadline-checking logic. E2E with `SAMETE_TEST_MODE=false` would require a separate server instance and is not worth the infrastructure complexity.

**Step 1:** Extract the deadline check into a testable function in `src/pages/home.rs`:
```rust
fn is_past_deadline(deadline: time::OffsetDateTime, test_mode: bool) -> bool {
    !test_mode && deadline < time::OffsetDateTime::now_utc()
}
```

**Step 2:** Use this function in both `enroll_in_season` and `confirm_ready`:
```rust
let test_mode = std::env::var("SAMETE_TEST_MODE").as_deref() == Ok("true");
if is_past_deadline(season.signup_deadline, test_mode) {
    return Err(ServerFnError::new("enrollment deadline has passed"));
}
```

**Step 3:** Write unit tests:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn past_deadline_blocks_when_not_test_mode() {
        let past = time::OffsetDateTime::now_utc() - time::Duration::hours(1);
        assert!(is_past_deadline(past, false));
    }

    #[test]
    fn past_deadline_allowed_in_test_mode() {
        let past = time::OffsetDateTime::now_utc() - time::Duration::hours(1);
        assert!(!is_past_deadline(past, true));
    }

    #[test]
    fn future_deadline_allowed_regardless() {
        let future = time::OffsetDateTime::now_utc() + time::Duration::hours(1);
        assert!(!is_past_deadline(future, false));
        assert!(!is_past_deadline(future, true));
    }
}
```

**Verification gates:**

```bash
# Gate 1: Function exists
grep -c 'fn is_past_deadline' src/pages/home.rs
# REQUIRED OUTPUT: 1

# Gate 2: Both callsites use it
grep -c 'is_past_deadline' src/pages/home.rs
# REQUIRED OUTPUT: at least 3 (definition + 2 call sites)

# Gate 3: Tests pass
cargo test is_past_deadline 2>&1 | tail -3
# REQUIRED OUTPUT: "test result: ok" with 3 tests passed
```

---

## Phase 5: Admin UI Analysis (Design Only)

**Problem:** The admin interface (5 pages) separates data from the actions it informs. The SMS page is the worst offender — it shows no target counts before triggering batch sends.

**Use the `dev-discipline:improve-architecture` skill.** Then use `frontend-design:frontend-design` for the proposed layout.

**Exploration findings (from recon):**

| Page | Data-Action Coupling |
|------|---------------------|
| Dashboard | DECOUPLED — health data shown, all actions elsewhere |
| Season | Self-contained — data and actions together |
| Participants | Self-contained |
| Assignments | Mostly coupled — confirmed count visible, swap/release available |
| SMS | CRITICALLY DECOUPLED — target counts invisible, organizer flies blind |

**Specific SMS decoupling:**
- `send_season_open_sms()` — cannot see how many active users will receive
- `send_assignment_sms()` — cannot see how many senders are unnotified
- `send_confirm_nudge_sms()` — cannot see how many are unconfirmed
- `send_receipt_nudge_sms()` — cannot see how many are non-responsive

**Analysis deliverable:** Write a proposal at `orchestration_log/recon/admin-redesign-proposal.md` covering:
1. Which pages merge, which stay separate
2. How SMS actions get target-count previews
3. How dashboard data gets co-located with its actions
4. Wireframe-level layout (ASCII is fine)

**Do NOT implement. The proposal requires user approval before any code changes.**

---

## Execution Order

```
Wave 1 (parallel — no file conflicts):
  Task 1A: Cancel confirmation dialog          [src/admin/season.rs]
  Task 1B: Cancelled-season HomeState           [src/pages/home.rs]
  Task 2B: 404 fallback test                    [src/app.rs, E2E files]
  Phase 3: Envelope reveal triage               [read-only investigation]
  Phase 4: Deadline enforcement tests           [src/pages/home.rs — different section than 1B]

Wave 2 (after Wave 1):
  Task 2A: Swap assignment E2E test             [src/admin/assignments.rs, E2E files]
  (depends on Wave 1 completing so E2E suite is stable before adding tests)

Wave 3 (after all implementation):
  qa-automation:executor-agent — run full suite, classify failures
  qa-automation:healer-agent — fix any broken locators
  Repeat until 2-3 consecutive green runs

Wave 4 (parallel with Waves 1-3):
  Phase 5: Admin UI analysis                    [read-only, write proposal only]

Wave 5 (after Phase 3 triage report):
  Envelope reveal re-implementation             [plan TBD based on triage findings]
```

---

## Forbidden Patterns

### BANNED: Skipping clippy after source changes
Every source file change requires `cargo clippy --all-targets --all-features -- -D warnings`. No exceptions.

### BANNED: Running E2E in the orchestrator
All E2E execution goes through `qa-automation:executor-agent` or delegated sonnet agents. The orchestrator never reads E2E output directly.

### BANNED: `waitForTimeout` in new E2E tests
Use element visibility waits or `clickAndWaitForResponse`. See `end2end/README.md`.

### BANNED: Implementing envelope fix before triage
Phase 3 produces a triage report. Implementation is a separate phase with its own plan.

### BANNED: Implementing admin redesign before proposal approval
Phase 5 produces a proposal. Implementation requires user sign-off.

### BANNED: Ad-hoc test runs for verification
Use `qa-automation:executor-agent` with structured failure classification. Not `just e2e-release` with manual output reading.

### BANNED: `format!()` constructing Tailwind class names
Per `guidance/frontend-protocol.md`. Use static class strings only.

### BANNED: Signal-driven form inputs for server function submission
Per `guidance/leptos-idioms.md`. Use ActionForm with `name` attributes.

---

## Definition of Done

1. [ ] `cargo clippy --all-targets --all-features -- -D warnings` produces zero output
2. [ ] `cargo test` passes all unit tests (including new `is_past_deadline` tests)
3. [ ] `qa-automation:executor-agent` reports 0 failures on full E2E suite (release build)
4. [ ] 2 additional consecutive green E2E runs (3 total)
5. [ ] Cancel button requires two clicks (confirmation step)
6. [ ] Participant home page shows distinct "season cancelled" state after cancellation
7. [ ] E2E test for swap assignment exists and passes
8. [ ] E2E test for 404 fallback exists and passes
9. [ ] Deadline enforcement has unit tests
10. [ ] Envelope reveal triage report exists at `orchestration_log/recon/envelope-triage.md`
11. [ ] Admin UI proposal exists at `orchestration_log/recon/admin-redesign-proposal.md`
12. [ ] All changes committed with conventional commit style
13. [ ] `deferred_items.md` updated to reflect closed items
14. [ ] `codebase_state.md` updated to reflect new test count and any new modules
