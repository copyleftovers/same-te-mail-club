# E2E Testing Reference — Leptos 0.8 + Playwright

Authoritative reference for E2E testing patterns in this project. Derived from Playwright docs, Leptos book, cargo-leptos internals, and project-specific battle-testing.

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Hydration Wait Strategy](#hydration-wait-strategy)
3. [ActionForm Submission Wait Strategy](#actionform-submission-wait-strategy)
4. [Playwright Config Recommendations](#playwright-config-recommendations)
5. [Reporter Configuration](#reporter-configuration)
6. [Port Management](#port-management)
7. [Serial Test Strategy](#serial-test-strategy)
8. [POM Improvements](#pom-improvements)
9. [Anti-Patterns](#anti-patterns)
10. [Leptos-Specific Gotchas](#leptos-specific-gotchas)
11. [Timeout Hierarchy](#timeout-hierarchy)

---

## Architecture Overview

### How the Test Runner Works

`cargo leptos end-to-end` is a thin wrapper that:
1. Builds the SSR binary and WASM client bundle
2. Starts the server on `site-addr` (127.0.0.1:3000)
3. Runs the shell command in `end2end-cmd` (`npx playwright test`) from `end2end-dir` (`end2end/`)
4. Kills the server when the command exits

The Playwright config has **no `webServer` block** — cargo-leptos owns the server lifecycle. This is intentional and correct.

### The Leptos Page Load Timeline

Understanding this timeline is critical for every test pattern:

```
1. Browser GETs the page
2. Server renders HTML (SSR) — complete, visible, but inert
3. Browser paints the SSR HTML immediately
4. Browser fetches the WASM bundle (~100KB–1MB depending on app)
5. WASM initializes, walks the existing DOM, attaches reactive signals and event listeners
6. Page is now interactive ("hydrated")
```

**The hydration gap** is the interval between steps 3 and 6. During this gap:
- The page *looks* interactive (buttons appear enabled, forms appear fillable)
- Event listeners are **not attached** — clicks do nothing
- Native HTML form submissions still work (progressive enhancement)

### This Project's Hydration Gate

Every component with an ActionForm uses a hydration gate pattern:

```rust
let (hydrated, set_hydrated) = signal(false);
Effect::new(move |_| {
    set_hydrated.set(true);
});

// Button is disabled until WASM hydrates
<button type="submit" disabled=move || !hydrated.get()>
```

Effects only run client-side after hydration. This means `disabled` is removed only after WASM attaches event listeners. **This is both a UX safety mechanism and a test synchronization mechanism.**

---

## Hydration Wait Strategy

### The Problem

Playwright's auto-waiting checks (visible, stable, enabled, receives-events) predate the SSR+hydration era. An SSR-rendered button passes all actionability checks *before* WASM hydration — it is visible, stable, enabled, and receives events at the DOM level. But clicking it does nothing because no JS event listener is attached yet.

### The Solution: Wait for `enabled` (already in place)

Because this project disables ActionForm buttons until hydration completes, Playwright's `enabled` actionability check naturally synchronizes with hydration. When a test calls:

```typescript
await page.getByTestId("launch-button").click();
```

Playwright auto-waits for the button to be enabled, which only happens after WASM hydrates and the `hydrated` signal fires. **No explicit hydration wait is needed for any button interaction — the disabled gate handles it.**

### For Non-Button Interactions

If a test needs to interact with elements that *don't* have a disabled gate (e.g., filling an input before clicking submit), the approach is:

```typescript
// Wait for ANY gated button on the page to become enabled — that proves hydration completed
await expect(page.getByRole("button", { name: /submit|save|send/i })).toBeEnabled();
// Now safe to fill inputs
await page.getByLabel(/phone/i).fill("+380670000001");
```

### Alternative: Custom Hydration Marker (if ever needed)

If a page has no gated buttons, you could add a global hydration marker. This is **not needed now** but documented for completeness:

**Rust side** (in the root App component):
```rust
let (hydrated, _) = signal(false);
Effect::new(move |_| {
    // Set a DOM attribute on <html> when WASM hydrates
    if let Some(doc) = document() {
        doc.document_element()
            .unwrap()
            .set_attribute("data-hydrated", "true")
            .ok();
    }
});
```

**Test side**:
```typescript
await page.waitForFunction(() =>
    document.documentElement.getAttribute("data-hydrated") === "true"
);
```

### Recommendation

**Do not add the global marker unless the button-disabled pattern proves insufficient.** The current pattern is simpler, requires no extra code, and is already deployed across all components.

---

## ActionForm Submission Wait Strategy

### The Problem

ActionForm submissions in Leptos 0.8 work differently depending on hydration state:

| State | Behavior |
|-------|----------|
| **Before WASM** | Native HTML form POST. Server processes, returns redirect (302) or HTML. Browser does full page navigation. |
| **After WASM** | ActionForm intercepts submit. Dispatches server action via fetch. Updates reactive signals. May trigger redirect via `leptos_axum::redirect()`. |

In tests, WASM is always hydrated by the time we click (thanks to the disabled gate). So the post-WASM path is what matters: **the click triggers an async fetch, then the page updates reactively.**

### The Core Rule

**Never assume the page has updated immediately after `.click()`.** Always wait for a **concrete UI signal** that proves the server function completed and the view re-rendered.

### Pattern 1: Wait for Element Appearance (preferred)

The best pattern is to wait for a UI element that only exists after the action completes:

```typescript
// Click submit
await page.getByTestId("create-season-button").click();
// Wait for the management panel that replaces the create form
await expect(page.getByTestId("launch-button")).toBeVisible();
```

This is already used in `createSeason()` and `launchSeason()` — it works correctly.

### Pattern 2: Wait for Element Disappearance

When the action causes the triggering element to vanish:

```typescript
await page.getByTestId("cancel-button").click();
await expect(page.getByTestId("cancel-button")).not.toBeVisible();
```

Already used in `cancelSeason()` — correct.

### Pattern 3: Wait for URL Change

When the server function issues a redirect:

```typescript
await page.getByRole("button", { name: /verify/i }).click();
await page.waitForURL("/");
```

Already used in `completeOnboarding()` and login flow.

### Pattern 4: Wait for Response (for actions with no visible UI change)

When there's no obvious UI indicator, wait for the HTTP response:

```typescript
const responsePromise = page.waitForResponse(
    (resp) => resp.url().includes("/api/") && resp.request().method() === "POST"
);
await page.getByTestId("advance-button").click();
const response = await responsePromise;
// Now assert on the updated page
```

**Important**: The `waitForResponse` promise must be created *before* the click. The response fires during the click, so setting up the listener after would miss it.

### Pattern 5: Wait for Navigation (for full-page ActionForm round-trips)

When an ActionForm triggers a redirect that causes full navigation:

```typescript
await Promise.all([
    page.waitForNavigation(),
    page.getByTestId("advance-button").click(),
]);
```

Note: Playwright's `.click()` already auto-waits for initiated navigations by default. This pattern is only needed if the auto-wait proves unreliable.

### The `advanceSeason` Problem

The current POM uses `waitForTimeout(1000)` after clicking advance:

```typescript
async advanceSeason() {
    await this.page.goto("/admin/season");
    await this.page.getByTestId("advance-button").click();
    await this.page.waitForTimeout(1000);  // BAD
}
```

**Fix options** (in order of preference):

1. **Wait for the advance button to re-enable** — after a successful advance, the page re-renders. If the button stays visible (because there are more phases to advance), it goes through disabled→enabled:
   ```typescript
   async advanceSeason() {
       await this.page.goto("/admin/season");
       const advanceBtn = this.page.getByTestId("advance-button");
       await advanceBtn.click();
       // After click, ActionForm disables the button (pending state),
       // then the resource refetches and re-renders.
       // Wait for the page to show updated phase content.
       // The caller's goToDashboard() + expectDashboardContent()
       // already validates the phase changed.
       await this.page.waitForLoadState("load");
   }
   ```

2. **Wait for the action's fetch response**:
   ```typescript
   async advanceSeason() {
       await this.page.goto("/admin/season");
       const responsePromise = this.page.waitForResponse(
           (resp) => resp.request().method() === "POST"
       );
       await this.page.getByTestId("advance-button").click();
       await responsePromise;
   }
   ```

3. **Wait for the pending state to resolve** — ActionForm shows "pending" text while the action runs, then re-renders:
   ```typescript
   async advanceSeason() {
       await this.page.goto("/admin/season");
       await this.page.getByTestId("advance-button").click();
       // Wait for the button to become enabled again (action completed, page re-rendered)
       await expect(this.page.getByTestId("advance-button")).toBeEnabled({ timeout: 10_000 });
   }
   ```

**Option 2 is recommended.** It is deterministic, does not depend on UI re-render timing, and directly proves the server function completed.

---

## Playwright Config Recommendations

```typescript
import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
    testDir: "./tests",

    // Test timeout: generous for SSR build + WASM hydration on first page load.
    // Individual assertions have their own expect timeout.
    timeout: 30_000,

    expect: {
        // 5s is adequate for most assertions.
        // Increase to 10s if hydration-dependent assertions are flaky.
        timeout: 5_000,
    },

    // Serial execution — tests share database state.
    fullyParallel: false,
    workers: 1,

    // Fail CI on .only, allow locally for debugging.
    forbidOnly: !!process.env.CI,

    // Serial retry: retries the ENTIRE serial group from the beginning.
    // 1 retry on CI catches transient hydration races without hiding real bugs.
    retries: process.env.CI ? 1 : 0,

    // Reporter: list for terminal visibility; html for post-mortem on failure.
    // Using array syntax to get both simultaneously.
    reporter: process.env.CI
        ? [["list"], ["html", { open: "never" }]]
        : [["list"], ["html", { open: "on-failure" }]],

    use: {
        baseURL: "http://127.0.0.1:3000",

        // Trace on first retry — captures network, DOM snapshots, console logs.
        // Invaluable for debugging flaky tests without reproducing locally.
        trace: "on-first-retry",

        // Screenshot on failure — cheaper than trace, always useful.
        screenshot: "only-on-failure",

        // Action timeout: prevent individual clicks from hanging forever.
        // 10s accommodates slow WASM hydration + server fn round-trip.
        actionTimeout: 10_000,

        // Navigation timeout: page.goto + waitForURL.
        // 15s accommodates cold-start SSR rendering.
        navigationTimeout: 15_000,
    },

    projects: [
        {
            name: "chromium",
            use: { ...devices["Desktop Chrome"] },
        },
    ],

    // NO webServer block. cargo-leptos manages the server.
});
```

### Key Changes from Current Config

| Setting | Current | Recommended | Rationale |
|---------|---------|-------------|-----------|
| `reporter` | `html` or `list` | Both via array | Always see terminal output; always get HTML report |
| `actionTimeout` | Not set (infinite) | `10_000` | Prevents hung tests on hydration failure |
| `navigationTimeout` | Not set (infinite) | `15_000` | Catches stuck navigation early |

---

## Reporter Configuration

### The Problem

The HTML reporter shows *nothing* in the terminal during a run. You stare at a blank screen until tests finish, then get a browser popup. This is terrible DX when running `just e2e`.

### Solution: Always Use List + HTML

```typescript
reporter: [
    ["list"],
    ["html", { open: "on-failure" }],
],
```

- `list`: Prints each test name + pass/fail in real-time to the terminal
- `html`: Generates a detailed report; opens browser automatically only when tests fail

### CI Override

```typescript
reporter: process.env.CI
    ? [["list"], ["html", { open: "never" }]]
    : [["list"], ["html", { open: "on-failure" }]],
```

On CI, the HTML report is generated (for artifact upload) but never auto-opens.

### Force Terminal Colors

If terminal output appears plain in some environments:

```bash
FORCE_COLOR=1 cargo leptos end-to-end
```

---

## Port Management

### The Problem

Stale `samete` processes from killed test runs linger on port 3000. The next `cargo leptos end-to-end` starts a new server, but if the old process already occupies the port, either:
- The new server fails to bind (explicit error — easy to diagnose)
- Playwright connects to the old server with stale DB state (silent failure — hard to diagnose)

### Solution: Kill Stale Processes Before E2E

Update the `justfile`:

```just
# Kill any stale samete processes on port 3000
_kill-stale:
    -lsof -i :3000 -t | xargs kill 2>/dev/null || true

# Run E2E tests via cargo-leptos + Playwright
e2e: _kill-stale db-reset db-seed
    SAMETE_TEST_MODE=true SAMETE_SMS_DRY_RUN=true cargo leptos end-to-end
```

The `-` prefix (or `|| true`) ensures the recipe doesn't fail if no process is found.

### Alternative: Use a Dedicated Test Port

Configure a test-specific port to avoid collisions with dev servers:

In `Cargo.toml` or via environment variable:
```toml
# Can't change site-addr per-profile, but cargo-leptos respects LEPTOS_SITE_ADDR
```

```just
e2e: _kill-stale db-reset db-seed
    SAMETE_TEST_MODE=true SAMETE_SMS_DRY_RUN=true LEPTOS_SITE_ADDR=127.0.0.1:3100 cargo leptos end-to-end
```

And in `playwright.config.ts`:
```typescript
use: {
    baseURL: process.env.LEPTOS_SITE_ADDR
        ? `http://${process.env.LEPTOS_SITE_ADDR}`
        : "http://127.0.0.1:3000",
},
```

**Recommendation**: Start with the kill-stale approach. Only add a dedicated port if collisions persist despite that.

---

## Serial Test Strategy

### Current Approach: `test.describe.serial`

Tests run in strict order within one describe block. Each test assumes the DB state left by the previous test. This is correct for this project because:

1. The test suite models a *narrative* — create season, launch, enroll, confirm, assign, deliver, complete
2. DB reset per test would require re-running the entire narrative prefix (O(n^2) total work)
3. The participant/season state machine has a natural linear progression

### The Retry Problem

With `test.describe.serial` and `retries: 1`:
- If test 15 fails, Playwright **restarts the entire serial group from test 1**
- This means the worker is killed, a fresh one starts, and all 40+ tests re-run
- This is expensive (~2 minutes for the full suite) but correct — partial retry would encounter wrong DB state

### Mitigation: Segment Serial Groups

Instead of one giant serial block, split into smaller serial groups with explicit setup:

```typescript
test.describe.serial("Epic 1: Auth", () => { /* ... */ });
test.describe.serial("Epic 4: Season", () => {
    test.beforeAll(async () => {
        // Explicit DB state assertion or setup
    });
    /* ... */
});
```

**However**, this only works if each group can establish its own preconditions. In this project, the groups are tightly coupled (Season depends on Auth, Enrollment depends on Season, etc.), so the single serial block is the right call.

### Recommendation: Keep the Single Serial Block

The current `test.describe.serial("The Mail Club", ...)` with inner describe blocks for readability is the correct architecture. Accept the retry cost. To reduce retry frequency, fix the root causes of flakiness (hydration waits, submission waits) rather than adding retries.

### Per-Group Retry Budget

If certain groups are flaky but others are stable:

```typescript
test.describe("Stable Group", () => {
    test.describe.configure({ retries: 0 });
    // ...
});

test.describe("Flaky Group", () => {
    test.describe.configure({ retries: 2 });
    // ...
});
```

This is **not recommended** for serial tests — it adds complexity without solving the core issue.

---

## POM Improvements

### Current POM Analysis

The `MailClubPage` POM is well-structured. Key improvements:

### 1. Fix `advanceSeason` — Replace `waitForTimeout`

```typescript
async advanceSeason() {
    await this.page.goto("/admin/season");
    const responsePromise = this.page.waitForResponse(
        (resp) => resp.request().method() === "POST"
    );
    await this.page.getByTestId("advance-button").click();
    await responsePromise;
}
```

### 2. Add Hydration Wait Helper

For pages where you need to ensure hydration before interacting with non-button elements:

```typescript
/**
 * Wait for WASM hydration to complete on the current page.
 * Relies on the project convention: ActionForm buttons start disabled
 * and become enabled after hydration.
 */
async waitForHydration() {
    // Find any submit button — they all use the disabled-until-hydrated pattern
    const submitBtn = this.page.locator('button[type="submit"]').first();
    // If there's a submit button, wait for it to be enabled
    if (await submitBtn.count() > 0) {
        await expect(submitBtn).toBeEnabled({ timeout: 10_000 });
    }
}
```

### 3. Strengthen `login()` with Explicit Navigation Wait

```typescript
async login(phone: string) {
    await this.page.goto("/login");

    // Wait for hydration (submit button becomes enabled)
    await expect(
        this.page.getByRole("button", { name: /send|submit|code/i })
    ).toBeEnabled();

    await this.page.getByLabel(/phone/i).fill(phone);
    await this.page.getByRole("button", { name: /send|submit|code/i }).click();

    // OTP step — wait for the code input to appear
    await expect(this.page.getByLabel(/code/i)).toBeVisible();
    await this.page.getByLabel(/code/i).fill(TEST_OTP);
    await this.page.getByRole("button", { name: /verify|submit|sign/i }).click();

    // Wait for navigation away from login
    await expect(this.page).not.toHaveURL(/\/login/);
}
```

Key changes:
- Explicit hydration wait before first fill
- Explicit wait for OTP step to appear before filling
- Explicit wait for navigation after verify (already implicit in callers, but better to have in POM)

### 4. Strengthen `registerParticipant()` with Submission Wait

```typescript
async registerParticipant(phone: string, name: string) {
    await this.page.goto("/admin/participants");
    // Wait for hydration
    await expect(this.page.getByTestId("register-button")).toBeEnabled();

    await this.page.getByLabel(/phone/i).fill(phone);
    await this.page.getByLabel(/name/i).fill(name);

    const responsePromise = this.page.waitForResponse(
        (resp) => resp.request().method() === "POST"
    );
    await this.page.getByTestId("register-button").click();
    await responsePromise;
}
```

### 5. Add Response-Wait Helper

Factor out the common "click and wait for POST response" pattern:

```typescript
/**
 * Click an element and wait for the server function response.
 * Use this for ActionForm submissions where you need to ensure
 * the server has processed the request before proceeding.
 */
async clickAndWaitForResponse(locator: Locator) {
    const responsePromise = this.page.waitForResponse(
        (resp) => resp.request().method() === "POST"
    );
    await locator.click();
    await responsePromise;
}
```

Then use it throughout:

```typescript
async advanceSeason() {
    await this.page.goto("/admin/season");
    await this.clickAndWaitForResponse(
        this.page.getByTestId("advance-button")
    );
}

async generateAssignments() {
    await this.page.goto("/admin/assignments");
    await this.clickAndWaitForResponse(
        this.page.getByTestId("generate-button")
    );
}
```

---

## Anti-Patterns

### 1. `waitForTimeout` — BANNED

```typescript
// BAD: arbitrary delay, flaky by definition
await page.waitForTimeout(1000);

// GOOD: wait for a concrete condition
await expect(page.getByTestId("advance-button")).toBeEnabled();
// OR
const resp = page.waitForResponse(r => r.request().method() === "POST");
await page.getByTestId("advance-button").click();
await resp;
```

`waitForTimeout` is the #1 source of both flakiness (too short on slow machines) and slowness (too long on fast machines). Every instance must be replaced with a deterministic wait.

### 2. `networkidle` — DISCOURAGED

```typescript
// BAD: unreliable, Playwright docs explicitly discourage it
await page.waitForLoadState("networkidle");

// GOOD: wait for the specific response you care about
await page.waitForResponse(r => r.url().includes("/api/"));
```

`networkidle` means "no network activity for 500ms." It's fragile because:
- WebSocket connections (like hot-reload) keep the network active
- Long-polling or streaming responses prevent idle
- It's a heuristic, not a semantic signal

### 3. Non-Retrying Assertions — BANNED

```typescript
// BAD: evaluates once, no retry
const text = await page.textContent(".status");
expect(text).toBe("Active");

// GOOD: auto-retries until timeout
await expect(page.locator(".status")).toHaveText("Active");
```

Playwright's web-first assertions (`expect(locator).toBeVisible()`, `.toHaveText()`, etc.) automatically retry. Raw `expect(value).toBe()` does not.

### 4. Checking Visibility Without Auto-Wait — BANNED

```typescript
// BAD: snapshot check, no retry
expect(await page.getByText("Active").isVisible()).toBe(true);

// GOOD: auto-retrying assertion
await expect(page.getByText("Active")).toBeVisible();
```

### 5. `force: true` on Click — BANNED (unless documented why)

```typescript
// BAD: bypasses actionability checks, hides real bugs
await page.getByTestId("button").click({ force: true });

// GOOD: fix the underlying issue (element overlapped, not visible, etc.)
await page.getByTestId("button").click();
```

### 6. Sleeping Before Assert — BANNED

```typescript
// BAD: race condition with extra steps
await page.click("#submit");
await page.waitForTimeout(500);
await expect(page.locator(".result")).toBeVisible();

// GOOD: the assertion auto-retries on its own
await page.click("#submit");
await expect(page.locator(".result")).toBeVisible();
```

---

## Leptos-Specific Gotchas

### 1. ActionForm Before Hydration = Native POST

If a user (or test) submits an ActionForm before WASM hydrates:
- The form submits as a native HTML POST
- The server processes it and returns either a redirect or raw response body
- The browser may show raw JSON or navigate unexpectedly

**This project prevents this** via the `disabled=move || !hydrated.get()` pattern. Tests naturally wait for the button to be enabled.

### 2. Hydration Mismatch = Silent DOM Corruption

If server-rendered HTML doesn't match what the client expects:
- Leptos walks the DOM tree and picks up wrong elements
- Event listeners attach to wrong nodes
- Clicks appear to do nothing, or affect the wrong element

Common causes:
- `cfg!(target_arch = "wasm32")` conditional rendering
- Invalid HTML (e.g., `<div>` inside `<p>`, missing `<tbody>`)
- Browser auto-correction creating extra elements

**Debugging**: Check browser console for hydration warnings. Validate HTML with W3C validator.

### 3. Resource Refetch Timing

After an ActionForm submission, the page updates via `Resource` refetch (triggered by `action.version()`). The refetch is async — the Resource fires a new server function call, waits for the response, then updates the DOM.

Timeline:
```
1. User clicks submit
2. ActionForm dispatches server action (POST)
3. Server processes, returns response
4. action.version() increments
5. Resource detects version change, fires refetch (GET)
6. Server returns fresh data
7. DOM updates with new data
```

This means **there are TWO round-trips** after a submit: the action POST and the resource refetch GET. Waiting for only the POST response is not sufficient if you need to assert on the refetched data. In that case, wait for the UI element that appears after the refetch completes.

### 4. Leptos Redirects in Server Functions

When a server function calls `leptos_axum::redirect("/some-path")`:
- **After hydration**: ActionForm intercepts the redirect and performs client-side navigation
- **Before hydration**: The browser follows the 302 redirect natively

In tests, this means `page.waitForURL("/some-path")` works in both cases.

### 5. `uuid::Uuid` SSR Panics

`Uuid::new_v4()` panics in WASM if the `js` feature is not enabled (no `crypto.getRandomValues` available). The project has `uuid/js` in the `hydrate` feature set. If you see a UUID-related panic in the browser console during E2E, check that this feature is present.

### 6. Leptos 0.8 Form Name Matching

ActionForm deserializes `FormData` by matching `name` attributes to server function parameters. A mismatch between the HTML `name="phone"` and the Rust parameter `phone: String` causes the server function to receive empty/default values silently.

When tests fail with "the server seems to ignore the form data," check name attributes first.

### 7. No `on:input` for E2E-Tested Forms

Playwright's `.fill()` does not reliably trigger Leptos `on:input` event handlers on hydrated elements. The synthetic input event doesn't propagate through Leptos's event delegation system the same way real user input does.

**Rule**: All forms tested by E2E must use ActionForm with `name` attributes. Never use `on:input` → signal → `action.dispatch()` for E2E-tested forms.

---

## Timeout Hierarchy

Playwright has a layered timeout system. Understanding it prevents both hung tests and premature failures.

```
Global Timeout (not set — entire run)
  └── Test Timeout (30s — one test function + beforeEach)
        ├── Action Timeout (10s recommended — per click/fill/etc)
        ├── Navigation Timeout (15s recommended — per goto/waitForURL)
        └── Expect Timeout (5s — per auto-retrying assertion)
```

### Recommended Values for This Project

| Timeout | Value | Rationale |
|---------|-------|-----------|
| `timeout` (test) | `30_000` | Generous for multi-step flows (login → navigate → act → assert) |
| `expect.timeout` | `5_000` | 5s for auto-retrying assertions; increase if hydration is slow |
| `actionTimeout` | `10_000` | Covers WASM hydration + click processing |
| `navigationTimeout` | `15_000` | Covers cold SSR page render |
| `globalTimeout` | Not set | Not needed for <50 tests |

### Per-Test Override (for Slow Tests)

```typescript
test("slow test", async ({ page }) => {
    test.setTimeout(60_000); // override test timeout
    // ...
    await expect(locator).toBeVisible({ timeout: 15_000 }); // override expect timeout
});
```

---

## Checklist: Before Merging New E2E Tests

- [ ] No `waitForTimeout` calls
- [ ] No `networkidle` waits
- [ ] Every ActionForm click has a concrete completion wait (element appears/disappears, URL changes, or response received)
- [ ] Every assertion uses web-first form (`expect(locator).toX()`, not `expect(await locator.x()).toBe()`)
- [ ] New POM methods include hydration wait where needed (button enabled check before first interaction)
- [ ] Test names trace to story numbers from `spec/User Stories.md`
- [ ] Test runs green 3 times in a row locally before claiming stable
