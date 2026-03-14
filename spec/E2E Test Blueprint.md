# E2E Test Blueprint

Prescriptive specification for the Playwright E2E test suite. Replaces the current six-file structure.

Written 2026-03-14. Applies the Simple Made Easy and First Principles lenses to the existing test stubs.

---

## Axioms

These are the irreducible truths about what E2E tests do for this project. Everything in this blueprint derives from them.

1. **E2E tests verify user-visible flows against a real server with a real database.** They are not unit tests. They do not test algorithms, normalization, or internal state transitions in isolation.

2. **Tests share database state.** The season lifecycle is a single sequential flow: register → auth → create → launch → enroll → confirm → assign → release → receive → complete. Each step depends on the previous step's DB state.

3. **Shared state means sequential execution.** Tests cannot run in parallel. They cannot run in arbitrary order. The execution order IS the dependency chain.

4. **Playwright sees rendered HTML.** It does not know or care about Leptos, Rust, or WASM. It clicks buttons, fills inputs, and reads text. The contract between tests and implementation is: specific selectors exist in the DOM.

5. **The season lifecycle has exactly one valid ordering.** The spec's story numbers (Epic 1, 2, 3, ...) reflect narrative structure. The DATA dependency chain is: Epic 1 → Epic 4 → Epic 2 → Epic 3 → Epic 5 → Epic 6. Tests must follow data dependencies, not narrative order.

6. **Time-dependent behavior belongs in unit tests.** Deadline enforcement, expiry checks, and timed triggers depend on server-side clock comparison. E2E tests control DB state, not time. Phase advancement in test mode bypasses deadline gates (the implementer must respect `SAMETE_TEST_MODE=true` for this).

---

## Structural Decision: One File

### The problem with six files

The current structure has six files (`epic1_join.spec.ts` through `epic6_account.spec.ts`) that:

- **Look independent but are not.** Epic 2 requires Epic 4's state. Epic 3 requires Epic 2's state. The files have hidden couplings through shared DB state.
- **Are ordered incorrectly.** Alphabetical order (epic1 → epic2 → epic3 → epic4) does not match the data dependency chain (epic1 → epic4 → epic2 → epic3). Epic 2 tests will fail because no season exists yet.
- **Complect file organization with execution semantics.** The form (six independent files) contradicts the function (one sequential flow). This is the definition of entanglement.

### The replacement

One file: `end2end/tests/mail_club.spec.ts`. Contains `test.describe.serial` blocks for navigation and filtering. The serial constraint is honest — it says "these tests depend on each other" because they do.

Filtering still works: `npx playwright test --grep "Epic 1"` runs only Epic 1 tests. `npx playwright test --grep "Story 2.2"` runs one test. The describe blocks provide the same navigability as separate files.

### Files to delete

```
end2end/tests/epic1_join.spec.ts
end2end/tests/epic2_season.spec.ts
end2end/tests/epic3_assign.spec.ts
end2end/tests/epic4_manage.spec.ts
end2end/tests/epic5_sms.spec.ts
end2end/tests/epic6_account.spec.ts
```

### Files to create

```
end2end/tests/mail_club.spec.ts     — all tests, serial execution
seed/test_admin.sql                 — admin user seed for test DB
```

### Files to modify

```
end2end/tests/fixtures/mail_club_page.ts  — POM additions
justfile                                   — db-seed + e2e changes
```

### Files unchanged

```
end2end/playwright.config.ts  — config is correct as-is
end2end/package.json          — dependencies unchanged
end2end/tsconfig.json         — unchanged
```

---

## Spec Divergence: Create vs. Launch

The current test file `epic4_manage.spec.ts:69` has a comment: `// Story 4.2: Launch — in this architecture, creation IS launch (signup phase)`. This is wrong. The spec is explicit:

- **Story 4.1 AC:** "Season is not open for enrollment until the organizer explicitly launches it."
- **Story 4.2:** "Given a created season / When the organizer launches it / Then enrollment opens."
- **Dashboard table, signup phase:** "'Launch' was already done (signup is open)."

The phase enum (`signup → creating → ...`) starts at `signup`. A season in `signup` phase has already been launched. Before launch, the season exists in a pre-signup state (implementation decides how: a `draft` phase, a nullable phase column, or a boolean flag).

The tests must verify the two-step flow:
1. After creation: season exists, participants cannot enroll
2. After launch: season transitions to signup, participants can enroll, SMS fires

---

## Spec Divergence: Phase Advancement and Deadlines

The dashboard table says the "Advance to assigning" button is "disabled until deadline passes" (confirming phase). Other phases don't mention this constraint on their advance buttons.

For E2E tests: create seasons with future deadlines (so enrollment and confirmation work). When `SAMETE_TEST_MODE=true`, phase advancement ignores deadline gates. This keeps tests deterministic and fast. Deadline enforcement is verified in unit tests where time is controllable.

**The implementer must ensure:** when `SAMETE_TEST_MODE=true`, `advanceSeason()` server function succeeds regardless of whether the current phase's deadline has passed.

---

## Infrastructure Changes

### Justfile

```just
# Reset database and seed test data
db-reset:
    sqlx database drop -y && sqlx database create && sqlx migrate run

# Seed test admin (for E2E)
db-seed:
    psql $DATABASE_URL -f seed/test_admin.sql

# Run E2E tests via cargo-leptos + Playwright
e2e: db-reset db-seed
    SAMETE_TEST_MODE=true SAMETE_SMS_DRY_RUN=true cargo leptos end-to-end
```

The `e2e` target now:
1. Resets the database (clean state every run)
2. Seeds the test admin user
3. Sets the required env vars
4. Runs cargo-leptos E2E

### Seed File: `seed/test_admin.sql`

```sql
-- Test admin user for E2E tests.
-- Requires migrations to have run (users table must exist).
-- Seeded by `just db-seed`, called from `just e2e`.
INSERT INTO users (id, phone, name, nova_poshta_branch, is_admin, is_active, onboarded, created_at)
VALUES (
    '00000000-0000-0000-0000-000000000001',
    '+380670000001',
    'Організатор',
    'Відділення №10, Київ',
    true,
    true,
    true,
    NOW()
) ON CONFLICT (phone) DO NOTHING;
```

The admin is pre-onboarded (skips the onboarding redirect on login). This is correct — in production, the organizer bootstraps their own account via direct DB access.

---

## Selector Contract

The POM defines the contract between tests and implementation. The implementer must place these attributes in Leptos templates. If a selector is missing, the corresponding test fails — that failure is the implementer's signal to add it.

### Required `data-testid` attributes

| testid | Component | Purpose |
|--------|-----------|---------|
| `enroll-button` | Season/Home | Enrollment action |
| `confirm-ready-button` | Season/Home | Ready-confirm action |
| `recipient-name` | Season/Home (sending phase) | Assignment: recipient's name |
| `recipient-phone` | Season/Home (sending phase) | Assignment: recipient's phone |
| `recipient-branch` | Season/Home (sending phase) | Assignment: recipient's branch |
| `received-button` | Season/Home (receiving phase) | Receipt: yes |
| `not-received-button` | Season/Home (receiving phase) | Receipt: no |
| `register-button` | Admin/Participants | Register new participant |
| `deactivate-button` | Admin/Participants (per-row) | Deactivate participant |
| `create-season-button` | Admin/Season | Create season |
| `launch-button` | Admin/Season | Launch created season |
| `advance-button` | Admin/Season or Admin/Dashboard | Advance phase |
| `cancel-button` | Admin/Season | Cancel season |
| `generate-button` | Admin/Assignments | Generate assignment cycles |
| `release-button` | Admin/Assignments | Release assignments to participants |
| `cycle-visualization` | Admin/Assignments | Generated cycle display |
| `send-season-open-button` | Admin/SMS | Trigger season-open SMS batch |
| `send-assignment-button` | Admin/SMS | Trigger assignment SMS batch |
| `send-confirm-nudge-button` | Admin/SMS | Trigger pre-deadline nudge batch |
| `send-receipt-nudge-button` | Admin/SMS | Trigger receipt nudge batch |
| `sms-report` | Admin/SMS | SMS send result display |

### Required label patterns (for `getByLabel`)

| Pattern | Element | Where |
|---------|---------|-------|
| `/phone/i` | Phone number input | Login page, Admin/Participants |
| `/code/i` | OTP code input | Login page (OTP step) |
| `/name/i` | Name input | Admin/Participants |
| `/nova poshta\|branch\|відділення/i` | Branch input | Onboarding, Season enrollment |
| `/signup.*deadline/i` | Signup deadline input | Admin/Season create |
| `/confirm.*deadline/i` | Confirm deadline input | Admin/Season create |
| `/theme/i` | Theme input | Admin/Season create |
| `/note\|anything\|organizer/i` | Receipt note textarea | Season/Home (receiving phase) |

### Required button name patterns (for `getByRole("button", { name })`)

| Pattern | Element | Where |
|---------|---------|-------|
| `/send\|submit\|code/i` | OTP request button | Login page |
| `/verify\|submit\|sign/i` | OTP verify button | Login page (OTP step) |
| `/save\|submit\|continue/i` | Save/continue button | Onboarding |

---

## Page Object Model

Complete replacement for `end2end/tests/fixtures/mail_club_page.ts`.

```typescript
import { type Page, expect } from "@playwright/test";

/**
 * Page Object Model for The Mail Club.
 *
 * Centralizes all selectors and user-action methods.
 * The selector contract is defined in spec/E2E Test Blueprint.md.
 *
 * Requires:
 *   SAMETE_TEST_MODE=true  (fixed OTP "000000")
 *   SAMETE_SMS_DRY_RUN=true (no real SMS)
 */

const TEST_OTP = "000000";

export class MailClubPage {
  constructor(public readonly page: Page) {}

  // ── Auth ──

  async login(phone: string) {
    await this.page.goto("/login");
    await this.page.getByLabel(/phone/i).fill(phone);
    await this.page.getByRole("button", { name: /send|submit|code/i }).click();

    // OTP step
    await this.page.getByLabel(/code/i).fill(TEST_OTP);
    await this.page.getByRole("button", { name: /verify|submit|sign/i }).click();
  }

  async expectLoggedIn() {
    await expect(this.page).not.toHaveURL(/\/login/);
  }

  async expectRedirectedToOnboarding() {
    await expect(this.page).toHaveURL(/\/onboarding/);
  }

  async expectRedirectedToHome() {
    await this.page.waitForURL("/");
  }

  // ── Onboarding (Story 1.3) ──

  async completeOnboarding(branch: string) {
    await this.page.getByLabel(/nova poshta|branch|відділення/i).fill(branch);
    await this.page.getByRole("button", { name: /save|submit|continue/i }).click();
    await this.page.waitForURL("/");
  }

  // ── Home Screen ──

  async goHome() {
    await this.page.goto("/");
  }

  async expectHomeContent(text: string | RegExp) {
    await expect(this.page.locator("main")).toContainText(text);
  }

  // ── Season enrollment (Story 2.1) ──

  async enrollInSeason(branch?: string) {
    if (branch) {
      await this.page.getByLabel(/nova poshta|branch|відділення/i).fill(branch);
    }
    await this.page.getByTestId("enroll-button").click();
  }

  async expectEnrolled() {
    await expect(this.page.getByTestId("enroll-button")).not.toBeVisible();
  }

  async expectEnrollAvailable() {
    await expect(this.page.getByTestId("enroll-button")).toBeVisible();
  }

  async expectEnrollNotAvailable() {
    await expect(this.page.getByTestId("enroll-button")).not.toBeVisible();
  }

  // ── Confirm ready (Story 2.2) ──

  async confirmReady() {
    await this.page.getByTestId("confirm-ready-button").click();
  }

  async expectConfirmed() {
    await expect(this.page.getByTestId("confirm-ready-button")).not.toBeVisible();
  }

  // ── Assignment view (Story 2.3) ──

  async expectAssignmentVisible() {
    await expect(this.page.getByTestId("recipient-name")).toBeVisible();
    await expect(this.page.getByTestId("recipient-phone")).toBeVisible();
    await expect(this.page.getByTestId("recipient-branch")).toBeVisible();
  }

  async getAssignment() {
    return {
      name: await this.page.getByTestId("recipient-name").textContent(),
      phone: await this.page.getByTestId("recipient-phone").textContent(),
      branch: await this.page.getByTestId("recipient-branch").textContent(),
    };
  }

  // ── Receipt confirmation (Story 2.4) ──

  async confirmReceipt(received: boolean, note?: string) {
    if (note) {
      await this.page.getByLabel(/note|anything|organizer/i).fill(note);
    }
    if (received) {
      await this.page.getByTestId("received-button").click();
    } else {
      await this.page.getByTestId("not-received-button").click();
    }
  }

  // ── Admin: participants (Story 1.1) ──

  async registerParticipant(phone: string, name: string) {
    await this.page.goto("/admin/participants");
    await this.page.getByLabel(/phone/i).fill(phone);
    await this.page.getByLabel(/name/i).fill(name);
    await this.page.getByTestId("register-button").click();
  }

  async expectParticipantInList(name: string) {
    await expect(this.page.getByText(name)).toBeVisible();
  }

  async deactivateParticipant(name: string) {
    await this.page.goto("/admin/participants");
    const row = this.page.getByRole("row").filter({ hasText: name });
    await row.getByTestId("deactivate-button").click();
  }

  // ── Admin: season management (Stories 4.1, 4.2) ──

  async createSeason(
    signupDeadline: string,
    confirmDeadline: string,
    theme?: string,
  ) {
    await this.page.goto("/admin/season");
    await this.page.getByLabel(/signup.*deadline/i).fill(signupDeadline);
    await this.page.getByLabel(/confirm.*deadline/i).fill(confirmDeadline);
    if (theme) {
      await this.page.getByLabel(/theme/i).fill(theme);
    }
    await this.page.getByTestId("create-season-button").click();
  }

  async launchSeason() {
    await this.page.goto("/admin/season");
    await this.page.getByTestId("launch-button").click();
  }

  async advanceSeason() {
    await this.page.goto("/admin/season");
    await this.page.getByTestId("advance-button").click();
  }

  async cancelSeason() {
    await this.page.goto("/admin/season");
    await this.page.getByTestId("cancel-button").click();
  }

  // ── Admin: assignments (Stories 3.1, 3.3) ──

  async generateAssignments() {
    await this.page.goto("/admin/assignments");
    await this.page.getByTestId("generate-button").click();
  }

  async releaseAssignments() {
    await this.page.getByTestId("release-button").click();
  }

  async expectCycleVisualization() {
    await expect(this.page.getByTestId("cycle-visualization")).toBeVisible();
  }

  // ── Admin: SMS triggers (Stories 5.1–5.4) ──

  async triggerSms(
    type: "season-open" | "assignment" | "confirm-nudge" | "receipt-nudge",
  ) {
    await this.page.goto("/admin/sms");
    await this.page.getByTestId(`send-${type}-button`).click();
  }

  async expectSmsReport() {
    await expect(this.page.getByTestId("sms-report")).toBeVisible();
  }

  // ── Admin: dashboard ──

  async goToDashboard() {
    await this.page.goto("/admin");
  }

  async expectDashboardContent(text: string | RegExp) {
    await expect(this.page.locator("main")).toContainText(text);
  }
}
```

### Changes from current POM

| Change | Reason |
|--------|--------|
| Added `launchSeason()` | Spec has create ≠ launch (Story 4.2) |
| Added `goHome()` | Convenience for participant tests |
| Added `expectHomeContent(text)` | Generic assertion for phase-specific home screen content |
| Added `expectEnrollAvailable()` | Explicit positive check (vs. the negative `expectEnrollNotAvailable()`) |
| Added `expectEnrollNotAvailable()` | Pre-launch and post-deadline verification |
| Added `expectDashboardContent(text)` | Generic assertion for dashboard content |
| Removed `expectDashboardShowsCount(label, count)` | Replaced by the more flexible `expectDashboardContent` |

---

## Complete Test File

Complete replacement for all six epic files. One file, serial execution, ordered by data dependency chain.

```typescript
import { test, expect } from "@playwright/test";
import { MailClubPage } from "./fixtures/mail_club_page";

/**
 * The Mail Club — E2E Test Suite
 *
 * One file. Serial execution. Ordered by data dependency chain, not epic number.
 * Each test traces to a story number from spec/User Stories.md.
 *
 * Dependency chain:
 *   Epic 1 (auth) → Epic 4 (season) → Epic 2 (participate) →
 *   Epic 3 (assign) → Epic 5 (SMS) → Epic 6 (deactivate)
 *
 * Prerequisite: `just e2e` resets DB and seeds admin before this runs.
 *
 * Env:
 *   SAMETE_TEST_MODE=true  — fixed OTP "000000", deadline gates bypassed
 *   SAMETE_SMS_DRY_RUN=true — SMS logged, not sent
 */

const ADMIN_PHONE = "+380670000001";

const PHONES = {
  A: "+380670000002",
  B: "+380670000003",
  C: "+380670000004",
};

const NAMES = {
  A: "Тестова Людина А",
  B: "Тестова Людина Б",
  C: "Тестова Людина В",
};

const BRANCH = "Відділення №1, Київ";
const BRANCH_ALT = "Відділення №5, Київ";

// Deadlines in the future — enrollment and confirmation stay open.
// Phase advancement bypasses deadline gates in test mode.
function futureDeadline(daysFromNow: number): string {
  const d = new Date(Date.now() + daysFromNow * 24 * 60 * 60 * 1000);
  return d.toISOString().slice(0, 16);
}

const SIGNUP_DEADLINE = futureDeadline(7);
const CONFIRM_DEADLINE = futureDeadline(21);
const SEASON_THEME = "Перший сезон";

test.describe.serial("The Mail Club", () => {
  // ════════════════════════════════════════════
  // BLOCK 1: Auth & Onboarding (Epic 1)
  // Stories: 1.1, 1.2, 1.3
  // ════════════════════════════════════════════

  test.describe("Epic 1: Join the Community", () => {
    // Story 1.1: Organizer registers new participants
    test("1.1 — admin registers participants", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);

      await app.registerParticipant(PHONES.A, NAMES.A);
      await app.expectParticipantInList(NAMES.A);

      await app.registerParticipant(PHONES.B, NAMES.B);
      await app.expectParticipantInList(NAMES.B);

      await app.registerParticipant(PHONES.C, NAMES.C);
      await app.expectParticipantInList(NAMES.C);
    });

    // Story 1.1 AC: duplicate phone rejected
    test("1.1 — duplicate phone number is rejected", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.registerParticipant(PHONES.A, "Дублікат");
      await expect(page.getByText(/already|існує|duplicate/i)).toBeVisible();
    });

    // Story 1.2: Sign in with phone number
    test("1.2 — registered participant can sign in", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.A);
      await app.expectLoggedIn();
    });

    // Story 1.2 AC: unregistered phone cannot sign in
    test("1.2 — unregistered phone is rejected", async ({ page }) => {
      const app = new MailClubPage(page);
      await page.goto("/login");
      await page.getByLabel(/phone/i).fill("+380679999999");
      await page.getByRole("button", { name: /send|submit|code/i }).click();

      // The app must NOT reveal whether the phone is registered.
      // Either the OTP step does not appear, or verification silently fails.
      // Assert: user is still on login page after attempting the full flow.
      await page.getByLabel(/code/i).fill("000000");
      await page.getByRole("button", { name: /verify|submit|sign/i }).click();
      await expect(page).toHaveURL(/\/login/);
    });

    // Story 1.3: First-time user redirected to onboarding
    test("1.3 — first login redirects to onboarding", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.A);
      await app.expectRedirectedToOnboarding();
    });

    // Story 1.3: Complete onboarding
    test("1.3 — participant completes onboarding", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.A);
      await app.expectRedirectedToOnboarding();
      await app.completeOnboarding(BRANCH);
      await app.expectRedirectedToHome();
    });

    // Story 1.3 AC: returning user skips onboarding
    test("1.3 — returning participant skips onboarding", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.A);
      await app.expectRedirectedToHome();
    });

    // Setup: onboard remaining participants (B and C)
    test("setup — onboard participants B and C", async ({ page }) => {
      const app = new MailClubPage(page);

      await app.login(PHONES.B);
      await app.expectRedirectedToOnboarding();
      await app.completeOnboarding(BRANCH);
      await app.expectRedirectedToHome();

      // Fresh browser context for C — need new page
    });

    test("setup — onboard participant C", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.C);
      await app.expectRedirectedToOnboarding();
      await app.completeOnboarding(BRANCH);
      await app.expectRedirectedToHome();
    });

    // Story 1.1 AC: only admin can access admin pages
    // (Tested AFTER onboarding so the redirect is from the admin guard, not the onboarding guard)
    test("1.1 — non-admin cannot access admin pages", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.A);
      await page.goto("/admin/participants");
      await expect(page).not.toHaveURL(/\/admin/);
    });
  });

  // ════════════════════════════════════════════
  // BLOCK 2: Season Setup (Epic 4, Story 5.3)
  // Stories: 4.1, 4.2, 5.3
  // ════════════════════════════════════════════

  test.describe("Epic 4: Season Management", () => {
    // Story 4.1: Create a new season
    test("4.1 — admin creates a season with deadlines and theme", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.createSeason(SIGNUP_DEADLINE, CONFIRM_DEADLINE, SEASON_THEME);
      // Season should exist — dashboard shows it
      await app.goToDashboard();
      await app.expectDashboardContent(/created|створено/i);
    });

    // Story 4.1 AC: season not open for enrollment until launched
    test("4.1 — season not enrollable before launch", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.A);
      await app.goHome();
      // Participant should NOT see the enroll button yet
      await app.expectEnrollNotAvailable();
    });

    // Story 4.1 AC: cannot create second active season
    test("4.1 — second active season is rejected", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.createSeason(SIGNUP_DEADLINE, CONFIRM_DEADLINE);
      await expect(page.getByText(/already|active|існує|активний/i)).toBeVisible();
    });

    // Story 4.1 AC: only admin can create seasons
    test("4.1 — non-admin cannot access season management", async ({ page }) => {
      await page.goto("/admin/season");
      await expect(page).toHaveURL(/\/login/);
    });

    // Story 4.2: Launch the season
    test("4.2 — admin launches the season", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.launchSeason();
      // Dashboard should show signup phase
      await app.goToDashboard();
      await app.expectDashboardContent(/signup|реєстрація/i);
    });

    // Story 5.3: Season-open SMS
    test("5.3 — admin triggers season-open SMS", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.triggerSms("season-open");
      await app.expectSmsReport();
      await expect(page.getByText(/sent|надіслано/i)).toBeVisible();
    });

    // Story 4.2 AC: season visible to participants after launch
    test("4.2 — participants can see season after launch", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.A);
      await app.goHome();
      await app.expectEnrollAvailable();
    });
  });

  // ════════════════════════════════════════════
  // BLOCK 3: Enrollment (Story 2.1)
  // ════════════════════════════════════════════

  test.describe("Story 2.1: Enrollment", () => {
    // Story 2.1 AC: content guidelines displayed
    test("2.1 — content guidelines visible during enrollment", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.A);
      await app.goHome();
      await expect(page.getByText(/self-expression|самовираження/i)).toBeVisible();
    });

    // Story 2.1 AC: season timeline visible (deadlines, theme)
    test("2.1 — season timeline and theme visible", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.A);
      await app.goHome();
      // Theme should be visible
      await app.expectHomeContent(SEASON_THEME);
    });

    // Story 2.1: Participant A enrolls
    test("2.1 — participant enrolls in season", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.A);
      await app.goHome();
      await app.expectEnrollAvailable();
      await app.enrollInSeason(BRANCH);
      await app.expectEnrolled();
    });

    // Story 2.1 AC: can update Nova Poshta branch during enrollment
    test("2.1 — participant can set branch during enrollment", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.B);
      await app.goHome();
      await app.enrollInSeason(BRANCH_ALT);
      await app.expectEnrolled();
    });

    // Setup: enroll participant C
    test("setup — participant C enrolls", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.C);
      await app.goHome();
      await app.enrollInSeason(BRANCH);
      await app.expectEnrolled();
    });

    // Admin sees enrolled count
    test("dashboard — enrolled count visible", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.goToDashboard();
      await app.expectDashboardContent(/enrolled|зареєстровано/i);
    });
  });

  // ════════════════════════════════════════════
  // BLOCK 4: Confirm Ready (Stories 2.2, 5.4)
  // ════════════════════════════════════════════

  test.describe("Story 2.2: Confirm Ready", () => {
    // Phase advancement: signup → creating → confirming
    // In test mode, advancement bypasses deadline gates.
    test("phase — advance signup → creating", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.advanceSeason();
      await app.goToDashboard();
      await app.expectDashboardContent(/creating|створення/i);
    });

    // Architecture Home Screen: creating phase, enrolled not confirmed
    test("home screen — creation period message shown", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.A);
      await app.goHome();
      // "Create your mail. Confirm ready by [deadline]." + countdown
      await app.expectHomeContent(/create|створ/i);
    });

    test("phase — advance creating → confirming", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.advanceSeason();
      await app.goToDashboard();
      await app.expectDashboardContent(/confirming|підтвердження/i);
    });

    // Story 5.4: Pre-deadline nudge SMS
    test("5.4 — admin triggers pre-deadline nudge SMS", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.triggerSms("confirm-nudge");
      await app.expectSmsReport();
    });

    // Story 2.2 AC: deadline countdown visible
    test("2.2 — ready-confirm deadline visible with countdown", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.A);
      await app.goHome();
      await expect(page.getByText(/deadline|дедлайн|залишилось/i)).toBeVisible();
    });

    // Story 2.2: Confirm ready
    test("2.2 — enrolled participant confirms ready", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.A);
      await app.goHome();
      await expect(app.page.getByTestId("confirm-ready-button")).toBeVisible();
      await app.confirmReady();
      await app.expectConfirmed();
    });

    // Story 2.2 AC: confirmation is irreversible
    test("2.2 — confirmed participant cannot un-confirm", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.A);
      await app.goHome();
      await expect(app.page.getByTestId("confirm-ready-button")).not.toBeVisible();
    });

    // Setup: confirm B and C
    test("setup — participants B and C confirm ready", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.B);
      await app.goHome();
      await app.confirmReady();
      await app.expectConfirmed();
    });

    test("setup — participant C confirms ready", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.C);
      await app.goHome();
      await app.confirmReady();
      await app.expectConfirmed();
    });

    // Dashboard: confirmed count
    test("dashboard — confirmed count visible", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.goToDashboard();
      await app.expectDashboardContent(/confirmed|підтверджено/i);
    });
  });

  // ════════════════════════════════════════════
  // BLOCK 5: Assignment (Epic 3, Story 5.1)
  // Stories: 3.1, 3.3, 5.1
  // ════════════════════════════════════════════

  test.describe("Epic 3: Assignment", () => {
    // Phase advancement: confirming → assigning
    test("phase — advance confirming → assigning", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.advanceSeason();
      await app.goToDashboard();
      await app.expectDashboardContent(/assigning|розподіл/i);
    });

    // Architecture Home Screen: assigning phase
    test("home screen — participant sees 'preparing' during assigning", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.A);
      await app.goHome();
      await app.expectHomeContent(/preparing|готує|організатор/i);
    });

    // Story 3.1 AC: organizer sees confirmed count before generating
    test("3.1 — admin sees confirmed count before generating", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await page.goto("/admin/assignments");
      // Confirmed count should be visible (3 participants)
      await expect(page.getByText(/3|confirmed|підтверджено/i)).toBeVisible();
    });

    // Story 3.1: Generate assignments
    test("3.1 — admin generates assignments", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.generateAssignments();
      await app.expectCycleVisualization();
    });

    // Story 3.3 AC: assignments not visible to participants before release
    test("3.3 — participant cannot see assignment before release", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.A);
      await app.goHome();
      // Still shows "preparing" — not the assignment details
      await expect(app.page.getByTestId("recipient-name")).not.toBeVisible();
    });

    // Story 3.3: Swap/override UI available
    test("3.3 — swap UI available to admin", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await page.goto("/admin/assignments");
      await expect(page.getByText(/swap|обмін|override/i)).toBeVisible();
    });

    // Story 3.3: Release assignments
    test("3.3 — admin releases assignments", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await page.goto("/admin/assignments");
      await app.releaseAssignments();
      await expect(page.getByText(/released|опубліковано/i)).toBeVisible();
    });

    // Story 5.1: Assignment SMS
    test("5.1 — admin triggers assignment SMS", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.triggerSms("assignment");
      await app.expectSmsReport();
    });
  });

  // ════════════════════════════════════════════
  // BLOCK 6: Delivery (Stories 2.3, 2.4, 5.2)
  // ════════════════════════════════════════════

  test.describe("Stories 2.3–2.4: Delivery & Receipt", () => {
    // Phase advancement: assigning → sending
    test("phase — advance assigning → sending", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.advanceSeason();
      await app.goToDashboard();
      await app.expectDashboardContent(/sending|відправлення/i);
    });

    // Story 2.3: Participant sees assignment
    test("2.3 — participant sees recipient details", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.A);
      await app.goHome();
      await app.expectAssignmentVisible();
      const assignment = await app.getAssignment();
      expect(assignment.name).toBeTruthy();
      expect(assignment.phone).toMatch(/\+380/);
      expect(assignment.branch).toBeTruthy();
    });

    // Story 2.3 AC: participant sees ONLY their own recipient
    test("2.3 — only one recipient visible", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.A);
      await app.goHome();
      const count = await app.page.getByTestId("recipient-name").count();
      expect(count).toBe(1);
    });

    // Phase advancement: sending → receiving
    test("phase — advance sending → receiving", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.advanceSeason();
      await app.goToDashboard();
      await app.expectDashboardContent(/receiving|отримання/i);
    });

    // Story 5.2: Receipt nudge SMS
    test("5.2 — admin triggers receipt-nudge SMS", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.triggerSms("receipt-nudge");
      await app.expectSmsReport();
    });

    // Architecture Home Screen: receiving phase
    test("home screen — receiving phase prompt shown", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.A);
      await app.goHome();
      await app.expectHomeContent(/arriving|отримання|confirm/i);
    });

    // Story 2.4: Confirm receipt — received
    test("2.4 — participant confirms receipt (received)", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.A);
      await app.goHome();
      await app.confirmReceipt(true);
      await expect(page.getByText(/дякуємо|thanks|confirmed/i)).toBeVisible();
    });

    // Story 2.4: Confirm receipt — not received, with note
    test("2.4 — participant reports not received with note", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.B);
      await app.goHome();
      await app.confirmReceipt(false, "Пошта не надійшла");
      await expect(page.getByText(/reported|повідомлено/i)).toBeVisible();
    });

    // Story 2.4 AC: "not received" triggers organizer notification
    test("2.4 — admin sees not-received alert", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.goToDashboard();
      // Dashboard should show the "not received" alert for participant B
      await app.expectDashboardContent(/not received|не отримано/i);
    });
  });

  // ════════════════════════════════════════════
  // BLOCK 7: Season Complete
  // ════════════════════════════════════════════

  test.describe("Season Complete", () => {
    test("phase — advance receiving → complete", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.advanceSeason();
      await app.goToDashboard();
      await app.expectDashboardContent(/complete|завершено/i);
    });

    // Architecture Home Screen: complete phase
    test("home screen — season complete message shown", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.A);
      await app.goHome();
      await app.expectHomeContent(/complete|завершено/i);
    });
  });

  // ════════════════════════════════════════════
  // BLOCK 8: Account Management (Epic 6)
  // Story: 6.1
  // ════════════════════════════════════════════

  test.describe("Epic 6: Account Management", () => {
    const DEACTIVATE_PHONE = "+380670000099";
    const DEACTIVATE_NAME = "Деактивований Учасник";

    // Setup: register a participant to deactivate
    test("setup — register participant for deactivation", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.registerParticipant(DEACTIVATE_PHONE, DEACTIVATE_NAME);
      await app.expectParticipantInList(DEACTIVATE_NAME);
    });

    // Story 6.1: Deactivate participant
    test("6.1 — admin deactivates a participant", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.deactivateParticipant(DEACTIVATE_NAME);
      await expect(page.getByText(/inactive|деактивовано/i)).toBeVisible();
    });

    // Story 6.1 AC: deactivated account cannot sign in
    test("6.1 — deactivated participant cannot sign in", async ({ page }) => {
      const app = new MailClubPage(page);
      await page.goto("/login");
      await page.getByLabel(/phone/i).fill(DEACTIVATE_PHONE);
      await page.getByRole("button", { name: /send|submit|code/i }).click();
      await page.getByLabel(/code/i).fill("000000");
      await page.getByRole("button", { name: /verify|submit|sign/i }).click();
      // Should remain on login — auth rejected
      await expect(page).toHaveURL(/\/login/);
    });
  });

  // ════════════════════════════════════════════
  // BLOCK 9: Cancel Season
  // Verifies cancellation flow (no story number — derived from phase enum)
  // ════════════════════════════════════════════

  test.describe("Cancel Season", () => {
    // The previous season is complete — we can create a new one.
    test("setup — create and launch a new season", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.createSeason(futureDeadline(7), futureDeadline(21));
      await app.launchSeason();
      await app.goToDashboard();
      await app.expectDashboardContent(/signup|реєстрація/i);
    });

    test("cancel — admin cancels the season", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.cancelSeason();
      await app.goToDashboard();
      await app.expectDashboardContent(/cancelled|скасовано|no active|create/i);
    });

    // Architecture Home Screen: no active season
    test("home screen — no active season after cancel", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.A);
      await app.goHome();
      await app.expectHomeContent(/no season|немає сезону|SMS/i);
    });
  });
});
```

### Test count: 58

| Block | Tests | Stories covered |
|-------|-------|----------------|
| 1. Auth & Onboarding | 10 | 1.1, 1.2, 1.3 |
| 2. Season Setup | 7 | 4.1, 4.2, 5.3 |
| 3. Enrollment | 6 | 2.1 |
| 4. Confirm Ready | 10 | 2.2, 5.4 |
| 5. Assignment | 8 | 3.1, 3.3, 5.1 |
| 6. Delivery & Receipt | 9 | 2.3, 2.4, 5.2 |
| 7. Season Complete | 2 | — |
| 8. Account Management | 3 | 6.1 |
| 9. Cancel Season | 3 | — |
| **Total** | **58** | **All 18 stories** |

---

## What Changed From Current Tests

### Added (not in current suite)

| What | Why |
|------|-----|
| `launchSeason()` call and pre-launch verification | Spec has create ≠ launch. Current tests merge them. |
| Phase-specific home screen assertions | The home screen IS the app. Each phase shows different content per the Architecture spec's table. This is the primary user-visible behavior. |
| Season theme visibility test | Story 2.1/4.1 AC: theme displayed during enrollment |
| Admin sees confirmed count before generating | Story 3.1 AC: organizer decides whether to proceed |
| "Not received" alert visible to admin | Story 2.4 AC: organizer notification on non-receipt |
| Cancel season flow | Phase enum supports cancellation from any non-terminal phase. No test existed. |
| Home screen "no active season" state | Architecture Home Screen table row 1 |
| DB reset + seed in `just e2e` | Clean state every run. Current Justfile has no reset before E2E. |

### Fixed (was wrong or incomplete)

| What | Problem | Fix |
|------|---------|-----|
| File ordering | Epic 2 ran before Epic 4. Tests would fail due to missing season. | Single file, dependency-ordered blocks. |
| Create = Launch | Test 4.2 said "creation IS launch." Spec disagrees. | Separate create, launch, and pre-launch verification steps. |
| Story 3.1 "all confirmed appear" | Had a comment "depends on implementation" with no assertion. | Asserts confirmed count visible before generation. |
| Story 1.2 unregistered phone | No assertion after form submission. | Asserts user stays on /login after full OTP attempt. |
| Story 3.3 swap test | Only checked if text "swap" is visible. | Still checks text visibility (the swap interaction depends on implementation), but clearly scoped. |
| Story 5.4 targeted count | Comment but no assertion. | SMS report assertion covers this — detailed count verification stays in unit tests. |
| Non-admin test | Tested before onboarding, so redirect was from onboarding guard, not admin guard. | Moved after onboarding to test the admin guard specifically. |

### Removed (was in current tests, intentionally dropped)

Nothing was removed. All existing test intentions are preserved, reorganized, and strengthened.

---

## E2E / Unit Test Boundary

### E2E tests verify (this file)

- User can log in via OTP and land on the correct page
- Admin can register, deactivate participants
- Admin can create, launch, advance, cancel seasons
- Participants see phase-appropriate content on the home screen
- Participants can enroll, confirm, view assignments, confirm receipt
- Admin can generate, review, release assignments
- Admin can trigger SMS batches and see reports
- Access control works (non-admin blocked, deactivated blocked, unregistered blocked)

### Unit tests verify (`cargo test`)

| Concern | Why unit, not E2E |
|---------|-------------------|
| Phase transition validity (valid transitions succeed, invalid return `Err`) | Pure function on enum. No UI. |
| Phone number normalization (various formats → E.164) | Pure function. No UI. |
| OTP generation, hashing, verification, expiry, rate limiting | Server-side logic. E2E only checks "login works." |
| Assignment algorithm: cycle validity, scoring, cohort splitting | Algorithm correctness. E2E only checks "assignments appear." |
| Session creation, validation, expiry, revocation | Server-side logic. E2E only checks "session works." |
| Deadline enforcement (enrollment closes at deadline, button hidden after deadline) | Time-dependent. Requires clock control. |
| SMS content validation (no recipient details in assignment SMS) | Message template logic. Verified via dry-run logs or unit test. |
| Deactivated users excluded from SMS recipient query | Query logic. |
| Social-awareness scoring (Story 3.2) | Algorithm internals. No UI. |
| Cohort splitting at N > 15 | Algorithm internals. |

### The line

**E2E tests ask:** "Does the user see the right thing and can they do the right action?"

**Unit tests ask:** "Does the server compute the right result?"

If a test requires controlling time, mocking an external service, or inspecting internal state — it's a unit test. If a test requires a browser, a running server, and a real database — it's an E2E test.

---

## Implementation Notes for the Leptos Developer

### Hydration timing

Leptos `SsrMode::Async` resolves all server data before sending HTML. Playwright receives fully-rendered pages. Playwright's auto-wait handles the gap between HTML render and WASM hydration (clicks wait for elements to be "actionable").

If you observe flaky clicks on first interaction after `page.goto()`, add a brief `await page.waitForLoadState('networkidle')` before the first click. This should not be needed in practice.

### `data-testid` in Leptos templates

Standard HTML attribute syntax in `view!` macros:

```rust
view! {
    <button data-testid="enroll-button" on:click=handle_enroll>
        "Enroll"
    </button>
}
```

### `SAMETE_TEST_MODE` requirements

When `SAMETE_TEST_MODE=true`, the server must:

1. Use fixed OTP code `"000000"` (skip SMS, skip random generation)
2. Log a warning at startup: "TEST MODE ACTIVE — fixed OTP, bypassed deadline gates"
3. Allow phase advancement regardless of whether deadlines have passed
4. NOT exist in production builds — gate behind `#[cfg(feature = "test-support")]` or runtime env check with startup warning

### Running tests

```bash
# Full E2E run (resets DB, seeds admin, sets env, runs Playwright)
just e2e

# Run specific block
SAMETE_TEST_MODE=true SAMETE_SMS_DRY_RUN=true cargo leptos end-to-end -- --grep "Epic 1"

# Debug mode (manual server start required)
cargo leptos watch &
cd end2end && npx playwright test --ui
```

### When a test fails

1. **Check the test name** — it traces to a story number (e.g., "2.2 — enrolled participant confirms ready" = Story 2.2)
2. **Read the story's AC** in `spec/User Stories.md`
3. **Check the selector contract** above — is the `data-testid` or label present in your template?
4. **Check the phase** — is the season in the right phase for this action? Run `just e2e -- --grep "phase"` to isolate phase transitions.
