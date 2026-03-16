# E2E Testing Guide — The Mail Club

Authoritative reference for writing, maintaining, and debugging Playwright E2E tests in this Leptos 0.8 project. Read this before touching any test file.

For deep-dive research and alternative patterns, see `ops/e2e-research.md`.

---

## Table of Contents

1. [Three Rules](#three-rules)
2. [Architecture](#architecture)
3. [The Hydration Gate](#the-hydration-gate)
4. [Wait Strategies](#wait-strategies)
5. [POM Contract](#pom-contract)
6. [Selector Contract](#selector-contract)
7. [Writing a New Test](#writing-a-new-test)
8. [Writing a New POM Method](#writing-a-new-pom-method)
9. [Test Structure](#test-structure)
10. [Banned Practices](#banned-practices)
11. [Running Tests](#running-tests)
12. [Debugging Failures](#debugging-failures)
13. [Pre-Merge Checklist](#pre-merge-checklist)
14. [Sources](#sources)

---

## Three Rules

Every E2E pattern in this project reduces to three rules:

1. **Wait for hydration before interacting.** Playwright's auto-wait handles this because all ActionForm buttons start `disabled` and become enabled only after WASM hydrates.

2. **Wait for the server function to complete after every ActionForm click.** Use `clickAndWaitForResponse()` — it sets up a POST response listener before clicking, so it cannot miss the response.

3. **Assert on concrete UI signals, not time.** Use `expect(locator).toBeVisible()`, `expect(page).toHaveURL()`, element appearance/disappearance. Never `waitForTimeout`. Never `networkidle`.

If you follow these three rules, your tests will be deterministic.

---

## Architecture

### Pipeline

```
just e2e
  ├── _kill-stale     (kill lingering processes on :3000)
  ├── db-reset        (drop, create, migrate)
  ├── db-seed         (insert test admin user)
  └── cargo leptos end-to-end
        ├── Build SSR binary + WASM client bundle
        ├── Start server on 127.0.0.1:3000
        ├── Run: npx playwright test (from end2end/)
        └── Kill server
```

**There is no `webServer` block in `playwright.config.ts`.** cargo-leptos owns the server lifecycle. This is intentional.

### Leptos Page Load Timeline

Every page load follows this sequence:

```
1. Browser GETs the URL
2. Server renders HTML (SSR) — complete, visible, but inert
3. Browser paints the SSR HTML immediately
4. Browser fetches the WASM bundle
5. WASM initializes, walks the DOM, attaches signals and event listeners
6. Page is now interactive ("hydrated")
```

**The hydration gap** (steps 3–6) is the danger zone: the page *looks* interactive but clicks do nothing. This project eliminates the gap through the hydration gate pattern.

### ActionForm Round-Trip

After WASM hydrates, ActionForm submissions follow this path:

```
1. User clicks submit button
2. ActionForm intercepts, dispatches server action via fetch (POST)
3. Server processes, returns response
4. action.version() increments
5. Any Resource using action.version() as source fires a refetch (GET)
6. DOM updates with new data
```

This means **two round-trips** after submit: the action POST and the resource refetch GET. `clickAndWaitForResponse()` waits for the POST. If you need data from the refetch, wait for the UI element that appears after the Resource re-renders.

---

## The Hydration Gate

Every component with an ActionForm uses this pattern:

```rust
let (hydrated, set_hydrated) = signal(false);
Effect::new(move |_| set_hydrated.set(true));

<button type="submit" disabled=move || !hydrated.get()>
```

Effects only run client-side after hydration. The button is disabled during SSR and the hydration gap, then enabled once WASM is live.

**Why this matters for tests:** Playwright auto-waits for buttons to be `enabled` before clicking. Because our buttons are disabled until hydration, `click()` naturally synchronizes with hydration. No explicit hydration wait is needed for button interactions.

**For non-button interactions** (filling inputs before the first click), wait for any submit button on the page to become enabled:

```typescript
await expect(page.getByTestId("some-submit-button")).toBeEnabled();
// Now safe to fill inputs
await page.getByLabel(/phone/i).fill("+380670000001");
```

---

## Wait Strategies

### After ActionForm Click → `clickAndWaitForResponse()`

The POM provides this helper. It is the **only** correct way to click an ActionForm submit button:

```typescript
async clickAndWaitForResponse(locator: Locator) {
    const responsePromise = this.page.waitForResponse(
        (resp) => resp.request().method() === "POST",
    );
    await locator.click();
    await responsePromise;
}
```

The listener is created **before** the click. This is critical — if you await the click first, the response may fire and be missed before you start listening.

### After Action Completes → Assert on UI

After `clickAndWaitForResponse`, the POST is done but the Resource refetch may still be in flight. Choose the right wait:

| Scenario | Wait Pattern |
|----------|-------------|
| New element appears after action | `await expect(locator).toBeVisible()` |
| Element disappears after action | `await expect(locator).not.toBeVisible()` |
| Page navigates (redirect) | `await page.waitForURL("/path")` |
| Text content changes | `await expect(locator).toContainText(/pattern/)` |
| No visible change (rare) | The POST wait from `clickAndWaitForResponse` is enough |

### After Navigation → Just Assert

`page.goto()` already waits for `load` by default. After navigation, directly assert:

```typescript
await this.page.goto("/admin");
await expect(this.page.locator("main")).toContainText(/dashboard/i);
```

---

## POM Contract

The POM (`tests/fixtures/mail_club_page.ts`) has two types of methods:

### Self-Contained Actions

These methods wait for their own completion. The test can proceed immediately after calling them.

| Method | Completion Signal |
|--------|------------------|
| `login(phone)` | Navigates away from `/login` |
| `completeOnboarding(branch)` | Navigates to `/` |
| `createSeason(signup, confirm, theme?)` | `launch-button` becomes visible |
| `launchSeason()` | `advance-button` becomes visible |
| `advanceSeason()` | POST response received |
| `cancelSeason()` | `cancel-button` disappears |
| `registerParticipant(phone, name)` | POST response received |

### Assertion-Separated Actions

These methods wait for the POST but leave UI verification to the caller. This allows tests to check for both success and failure paths.

| Method | What it waits for | Caller should assert |
|--------|------------------|---------------------|
| `enrollInSeason(branch?)` | POST response | `expectEnrolled()` or error text |
| `confirmReady()` | POST response | `expectConfirmed()` or error text |
| `confirmReceipt(received, note?)` | POST response | Thank-you/reported text |
| `generateAssignments()` | POST response | `expectCycleVisualization()` |
| `releaseAssignments()` | POST response | Released text |
| `triggerSms(type)` | POST response | `expectSmsReport()` |
| `deactivateParticipant(name)` | POST response | Deactivated text |

### Pure Assertions

These methods only check state. They auto-retry via Playwright's web-first assertions.

`expectLoggedIn()`, `expectRedirectedToOnboarding()`, `expectRedirectedToHome()`, `expectHomeContent()`, `expectEnrolled()`, `expectEnrollAvailable()`, `expectEnrollNotAvailable()`, `expectConfirmed()`, `expectAssignmentVisible()`, `expectCycleVisualization()`, `expectSmsReport()`, `expectDashboardContent()`, `expectParticipantInList()`

---

## Selector Contract

Tests find elements via three mechanisms, in order of preference:

1. **`data-testid` attributes** — for elements specific to test interaction. Used via `page.getByTestId("enroll-button")`. The Rust component sets `data-testid="enroll-button"` on the element.

2. **Accessible roles and labels** — for standard form elements. Used via `page.getByLabel(/phone/i)` or `page.getByRole("button", { name: /send/i })`. The Rust component uses `<label>` elements or `aria-label` attributes.

3. **Text content** — for asserting visible content. Used via `page.getByText(/pattern/i)` or `expect(locator).toContainText()`. Patterns are case-insensitive and bilingual (Ukrainian + English) to tolerate i18n changes.

**Bilingual patterns:** This app uses Ukrainian UI text. All regex patterns must match both Ukrainian and English variants: `/enrolled|зареєстровано/i`. This prevents tests from breaking when translations change.

**Adding new test IDs:** When implementing a Rust component that tests will interact with, add `data-testid="descriptive-name"` to actionable elements (buttons, links) and key display elements (status text, data fields).

---

## Writing a New Test

### Template

```typescript
test("X.Y — description matching story AC", async ({ page }) => {
    const app = new MailClubPage(page);
    // 1. Authenticate (or reuse existing session)
    await app.login(PHONES.A);
    // 2. Navigate to the right page
    await app.goHome();
    // 3. Perform action via POM
    await app.enrollInSeason(BRANCH);
    // 4. Assert outcome via POM or web-first assertion
    await app.expectEnrolled();
});
```

### Rules

- **Test name** traces to a story number: `"2.1 — participant enrolls in season"`.
- **One logical assertion per test.** Multiple `expect` calls are fine if they verify the same thing (e.g., checking three fields of an assignment).
- **Use the POM.** Never write raw selectors in the test file unless testing selector behavior itself.
- **Web-first assertions only.** Always `await expect(locator).toX()`, never `expect(await locator.x()).toBe()`.
- **No test-specific waits.** If you need a new wait pattern, add it to the POM.
- **Setup tests are prefixed** `"setup — "` and exist only to establish DB state for subsequent tests. They are not stories.

---

## Writing a New POM Method

### For ActionForm Submissions

```typescript
async doSomething() {
    await this.page.goto("/the-page");
    // Wait for hydration if filling inputs before the first click.
    await expect(this.page.getByTestId("submit-button")).toBeEnabled();
    // Fill form fields.
    await this.page.getByLabel(/field/i).fill("value");
    // Click and wait for server function.
    await this.clickAndWaitForResponse(
        this.page.getByTestId("submit-button"),
    );
    // Optionally wait for a UI completion signal if this is a self-contained action.
    await expect(this.page.getByTestId("result")).toBeVisible();
}
```

### For Navigation + Assertion

```typescript
async goToSomePage() {
    await this.page.goto("/some-page");
}

async expectSomeContent(text: string | RegExp) {
    await expect(this.page.locator("main")).toContainText(text);
}
```

### Checklist for New POM Methods

- [ ] Uses `clickAndWaitForResponse()` for every ActionForm button click
- [ ] Waits for hydration (`toBeEnabled()`) if filling inputs before first click
- [ ] Self-contained actions wait for their completion signal
- [ ] Assertion-separated actions document what the caller should assert
- [ ] All selectors use testids, roles, or labels — never CSS classes or tag structures

---

## Test Structure

### Serial Execution

All tests run in a single `test.describe.serial("The Mail Club", ...)` block. Tests share database state — each test depends on the DB state left by previous tests.

**Why serial:** The test suite models a narrative (register → login → onboard → create season → enroll → confirm → assign → deliver → complete). Resetting the DB per test would require replaying the entire prefix, making the suite O(n²).

**Consequence of failure:** If test N fails, all tests after N are skipped (they would encounter wrong DB state). On CI with `retries: 1`, Playwright restarts the entire serial group from test 1.

### Block Organization

Tests are grouped into `test.describe` blocks by epic/story for readability:

```
Epic 1: Auth & Onboarding
Epic 4: Season Management
Story 2.1: Enrollment
Story 2.2: Confirm Ready
Epic 3: Assignment
Stories 2.3–2.4: Delivery & Receipt
Season Complete
Epic 6: Account Management
Cancel Season
```

**Block order follows the data dependency chain**, not epic numbers.

---

## Banned Practices

### `waitForTimeout` — BANNED

Arbitrary delays are flaky by definition. Too short on slow machines, too slow on fast ones. Use a deterministic wait.

```typescript
// WRONG
await page.waitForTimeout(1000);

// RIGHT
await this.clickAndWaitForResponse(button);
// or
await expect(locator).toBeVisible();
```

### `networkidle` — BANNED

Playwright's own docs discourage it. It means "no network activity for 500ms" — fragile with WebSockets, long-polling, or any background fetch.

### Non-Retrying Assertions — BANNED

```typescript
// WRONG — evaluates once, no retry
const text = await page.textContent(".status");
expect(text).toBe("Active");

// RIGHT — auto-retries until timeout
await expect(page.locator(".status")).toHaveText("Active");
```

### `force: true` on Click — BANNED

Bypasses all actionability checks. If the button isn't clickable, there's a real bug — find it.

### Raw Selectors in Test Files — BANNED

All selectors live in the POM. Tests use POM methods.

### `page.evaluate` for Assertions — BANNED

Use Playwright's web-first assertions. `page.evaluate` runs once with no retry.

---

## Running Tests

```bash
# Full pipeline (kill stale, reset DB, seed, build, test)
just e2e

# Only run Playwright (server must already be running)
cd end2end && npx playwright test

# Run a specific test by title
cd end2end && npx playwright test -g "2.1 — participant enrolls"

# Run with headed browser (see what's happening)
cd end2end && npx playwright test --headed

# Run with Playwright debug UI (step through)
cd end2end && npx playwright test --debug
```

### Reporter Output

The config uses both `list` (terminal) and `html` (browser) reporters. You always see test names and pass/fail in the terminal. On failure, the HTML report opens automatically for screenshots, traces, and DOM snapshots.

---

## Debugging Failures

### Test Hangs or Times Out

1. **Check for stale processes:** `lsof -i :3000`. Kill them: `lsof -i :3000 -t | xargs kill`.
2. **Check the server started:** Look for the build output before Playwright starts. If the build failed, Playwright connects to nothing or a stale server.
3. **Check actionTimeout:** If a click times out, the button is probably still `disabled` — meaning WASM didn't hydrate. Check the browser console for WASM errors or hydration mismatches.

### Test Fails on Assertion

1. **Run with `--headed`** to see the actual page state at failure time.
2. **Check the HTML report** for screenshots and DOM snapshots.
3. **Check the trace** (generated on first retry): `npx playwright show-trace trace.zip`.
4. **Check for hydration mismatch:** Open browser console during headed run. Leptos logs hydration warnings. Invalid HTML (e.g., `<div>` inside `<p>`) causes the DOM walker to attach signals to wrong nodes.

### Tests Pass Locally, Fail on CI

1. **Timing:** CI machines are slower. If a test relies on timing (it shouldn't, but), increase timeouts.
2. **DB state:** Check that `just e2e` runs the full pipeline (kill-stale + db-reset + db-seed).
3. **Port collision:** CI may have parallel jobs. Consider a dedicated test port via `LEPTOS_SITE_ADDR`.

### Flaky Test (Passes on Retry)

A flaky test is a test with a missing wait. Find the missing synchronization point:
1. Identify the action that precedes the failing assertion.
2. Check if the POM method uses `clickAndWaitForResponse()`.
3. Check if the assertion waits for a concrete UI signal (not time).
4. If both are correct, the issue is likely a Resource refetch race — add a UI element wait after the POST wait.

---

## Pre-Merge Checklist

- [ ] No `waitForTimeout` calls anywhere in POM or tests
- [ ] No `networkidle` waits
- [ ] Every ActionForm click uses `clickAndWaitForResponse()`
- [ ] Every assertion uses web-first form (`await expect(locator).toX()`)
- [ ] New POM methods wait for hydration where needed
- [ ] Selectors use testids/roles/labels, not CSS classes
- [ ] Test names trace to story numbers
- [ ] Bilingual regex patterns for all text assertions
- [ ] Tests pass 3 times in a row locally before claiming stable

---

## Sources

### Playwright

- [Auto-waiting & actionability checks](https://playwright.dev/docs/actionability) — how Playwright decides when an element is ready for interaction. Our `disabled` hydration gate exploits the "enabled" check.
- [Web-first assertions](https://playwright.dev/docs/test-assertions) — why `expect(locator).toBeVisible()` retries but `expect(value).toBe()` does not.
- [page.waitForResponse()](https://playwright.dev/docs/api/class-page#page-wait-for-response) — the API behind `clickAndWaitForResponse()`. Must be set up before the triggering action.
- [Test reporters](https://playwright.dev/docs/test-reporters) — array syntax for multiple simultaneous reporters.
- [Serial mode](https://playwright.dev/docs/test-parallel#serial-mode) — how `test.describe.serial` handles retries (restarts entire group).
- [Best practices](https://playwright.dev/docs/best-practices) — Playwright team's official guidance. Confirms: no `networkidle`, no `waitForTimeout`, prefer web-first assertions.

### Leptos

- [Server functions](https://book.leptos.dev/server/25_server_functions.html) — how ActionForm dispatches to server functions via POST.
- [Hydration](https://book.leptos.dev/ssr/24_hydration.html) — the SSR → WASM handoff process. Explains why the hydration gap exists.
- The project's Leptos MCP server (`mcp__plugin_leptos-mcp_leptos__get-documentation`) provides authoritative Leptos 0.8 docs. Query the `forms-and-actions` and `mental-model` sections for ActionForm and hydration details.

### Project-Specific

- `ops/e2e-research.md` — deep-dive research document with alternative patterns, timeout hierarchy, and Leptos-specific gotchas.
- `ops/leptos-idioms.md` — Leptos 0.8 patterns used in this codebase, including the hydration gate and ActionForm conventions.
- `spec/E2E Test Blueprint.md` — the selector contract between Rust components and Playwright tests.
- `spec/User Stories.md` — acceptance criteria that tests are derived from. Every test traces to a story number.
