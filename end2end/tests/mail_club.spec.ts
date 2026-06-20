import { test, expect } from "./fixtures/cached-context";
import { MailClubPage } from "./fixtures/mail_club_page";

/**
 * The Mail Club — E2E Test Suite
 *
 * One file. Serial execution. Ordered by data dependency chain, not epic number.
 * Each test traces to a story number from spec/User Stories.md.
 *
 * Dependency chain (main serial block):
 *   Epic 1 (auth) → Epic 4 (season) → Epic 2 (participate) →
 *   Epic 3 (assign) → Epic 5 (SMS) → Cancel Season
 *
 * Independent serial blocks (run even if main chain fails):
 *   Account Management (Epic 6) — creates its own participant
 *   Session Management (Logout) — only needs seeded admin + Epic 1 users
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

// Extra phones for edge-case rejection tests (invite code flow only)
const EXTRA_PHONES = {
  INVALID_CODE_TEST: "+380670000005",
  USED_CODE_TEST: "+380670000006",
  REVOKED_CODE_TEST: "+380670000007",
};

const NAMES = {
  A: "Тестова Людина А",
  B: "Тестова Людина Б",
  C: "Тестова Людина В",
};

const CITY_KYIV = "Київ";
const CITY_LVIV = "Львів";
const BRANCH_1 = "1";
const BRANCH_5 = "5";
const BRANCH_10 = "10";

// Deadlines in the future — enrollment and confirmation stay open.
// Phase advancement bypasses deadline gates in test mode.
function futureDeadline(daysFromNow: number): string {
  const d = new Date(Date.now() + daysFromNow * 24 * 60 * 60 * 1000);
  return d.toISOString().slice(0, 16);
}

const SIGNUP_DEADLINE = futureDeadline(7);
const CONFIRM_DEADLINE = futureDeadline(21);
const SEASON_THEME = "Перший сезон";

// Invite codes generated during the test run.
// Populated in "1.5 — admin generates invite codes" and consumed by subsequent
// self-registration and rejection tests. Module-level so serial tests share state.
const CODES: { A: string; B: string; C: string; DEACTIVATE: string; REVOKED: string } = {
  A: "",
  B: "",
  C: "",
  DEACTIVATE: "",
  REVOKED: "",
};

test.describe.serial("The Mail Club", () => {
  // ════════════════════════════════════════════
  // BLOCK 1: Auth & Onboarding (Epic 1)
  // Stories: 1.1, 1.2, 1.3, 1.5, 1.6
  // ════════════════════════════════════════════

  test.describe("Epic 1: Join the Community", () => {
    // Story 1.5: Admin generates invite codes for participants
    test("1.5 — admin generates invite codes for participants", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);

      // Generate three codes for the main participants (A, B, C).
      // In a freshly seeded DB the only active user is the admin — the
      // distributor dropdown auto-selects the first (only) available option.
      CODES.A = await app.generateInviteCode();
      CODES.B = await app.generateInviteCode();
      CODES.C = await app.generateInviteCode();

      // Codes must be non-empty strings.
      expect(CODES.A.length).toBeGreaterThan(0);
      expect(CODES.B.length).toBeGreaterThan(0);
      expect(CODES.C.length).toBeGreaterThan(0);
    });

    // Story 1.5 AC: generated codes appear in admin list with unused status
    test("1.5 — generated codes appear in admin list with unused status", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await page.goto("/admin");

      // All three codes from the previous test should be visible as unused.
      await app.expectInviteCodeStatus(CODES.A, "unused");
      await app.expectInviteCodeStatus(CODES.B, "unused");
      await app.expectInviteCodeStatus(CODES.C, "unused");
    });

    // Story 1.6: Admin revokes an unused code (4th code — used for revocation test)
    test("1.6 — admin revokes an unused code", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);

      // Generate a 4th code specifically to revoke.
      CODES.REVOKED = await app.generateInviteCode();
      expect(CODES.REVOKED.length).toBeGreaterThan(0);

      // Revoke it.
      await app.revokeInviteCode(CODES.REVOKED);

      // Status must change to revoked.
      await app.expectInviteCodeStatus(CODES.REVOKED, "revoked");
    });

    // Story 1.1: Participant A self-registers with invite code
    test("1.1 — participant A self-registers with invite code", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.selfRegister(PHONES.A, CODES.A, NAMES.A);
      // selfRegister completes at /onboarding (first-time login requires onboarding).
      await app.expectRedirectedToOnboarding();
    });

    // Story 1.1 AC: invalid invite code is rejected
    test("1.1 — invalid invite code is rejected", async ({ page }) => {
      const app = new MailClubPage(page);

      // Use a phone not in the system to reach the invite code step.
      await app.reachInviteCodeStep(EXTRA_PHONES.INVALID_CODE_TEST);
      await page.getByTestId("invite-code-input").fill("invalid-garbage-code");
      await page.getByTestId("submit-invite-code-button").click();

      // Error is displayed; the form stays on the invite code step.
      await expect(page.getByTestId("invite-code-error")).toBeVisible();
      // Name collection step must NOT have appeared.
      await expect(page.getByTestId("legal-name-input")).not.toBeVisible();
    });

    // Story 1.6 AC: revoked code cannot be redeemed
    test("1.6 — revoked code cannot be redeemed", async ({ page }) => {
      const app = new MailClubPage(page);

      await app.reachInviteCodeStep(EXTRA_PHONES.REVOKED_CODE_TEST);
      await page.getByTestId("invite-code-input").fill(CODES.REVOKED);
      await page.getByTestId("submit-invite-code-button").click();

      // The revoked code must be rejected with an error.
      await expect(page.getByTestId("invite-code-error")).toBeVisible();
      await expect(page.getByTestId("legal-name-input")).not.toBeVisible();
    });

    // Story 1.1 AC: used invite code is rejected on second attempt
    test("1.1 — used invite code is rejected", async ({ page }) => {
      const app = new MailClubPage(page);

      // CODES.A was used by participant A in a previous test.
      // Try to redeem it with a different phone.
      await app.reachInviteCodeStep(EXTRA_PHONES.USED_CODE_TEST);
      await page.getByTestId("invite-code-input").fill(CODES.A);
      await page.getByTestId("submit-invite-code-button").click();

      // Code was already used — must show error.
      await expect(page.getByTestId("invite-code-error")).toBeVisible();
      await expect(page.getByTestId("legal-name-input")).not.toBeVisible();
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
      await app.attemptLogin("+380679999999");
      // The app must NOT reveal whether the phone is registered.
      // Either the OTP step does not appear, or verification silently fails.
      // Assert: user is still on login page after attempting the full flow.
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
      await app.completeOnboarding(CITY_KYIV, BRANCH_1);
      await app.expectRedirectedToHome();
    });

    // Story 1.3 AC: returning user skips onboarding
    test("1.3 — returning participant skips onboarding", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.A);
      await app.expectRedirectedToHome();
    });

    // Setup: self-register and onboard participant B
    test("setup — self-register participant B", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.selfRegister(PHONES.B, CODES.B, NAMES.B);
      await app.expectRedirectedToOnboarding();
      await app.completeOnboarding(CITY_KYIV, BRANCH_1);
      await app.expectRedirectedToHome();
    });

    // Setup: self-register and onboard participant C
    test("setup — self-register participant C", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.selfRegister(PHONES.C, CODES.C, NAMES.C);
      await app.expectRedirectedToOnboarding();
      await app.completeOnboarding(CITY_KYIV, BRANCH_1);
      await app.expectRedirectedToHome();
    });

    // Story 1.1 AC: only admin can access admin pages
    // (Tested AFTER onboarding so the redirect is from the admin guard, not the onboarding guard)
    test("1.1 — non-admin cannot access admin pages", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.A);
      await page.goto("/admin");
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
      // Season should exist — dashboard shows the launch button in Created phase.
      // Neither "created" nor "створено" appears as text; the Created phase renders
      // the phase-stepper and the "Запустити" launch button. (FIX-1)
      await app.goToDashboard();
      await app.expectDashboardContent(/запустити/i);
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
    // When an active season exists, the create form is hidden — the page shows
    // the management panel instead. No second season can be created.
    test("4.1 — second active season is rejected", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await page.goto("/admin");
      // Create form should NOT be visible (season already exists)
      await expect(page.getByTestId("create-season-button")).not.toBeVisible();
      // Management panel should be visible instead
      await expect(page.getByTestId("launch-button")).toBeVisible();
    });

    // Story 4.1 AC: only admin can create seasons
    test("4.1 — non-admin cannot access season management", async ({ page }) => {
      await page.goto("/admin");
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

    // Story 4.5: All admin functionality on /admin (NEW-4.5-A)
    // Dependencies: admin logged in, season launched (Enrollment phase).
    test("4.5 — all admin functionality on /admin", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await page.goto("/admin");
      await expect(page.locator("main")).toBeVisible({ timeout: 15000 });

      // Season section visible
      await expect(page.getByTestId("dashboard-content")).toBeVisible();

      // Participants section visible on same page
      await expect(page.getByTestId("participant-list")).toBeVisible();

      // Invite code generation section visible (replaced old register form)
      await expect(page.getByTestId("generate-code-button")).toBeVisible();

      // Phase controls visible — Enrollment phase shows the season-open SMS button
      await expect(page.getByTestId("send-season-open-button")).toBeVisible();
    });

    // Story 4.5: Phase-stepper present in all launched phases (NEW-4.5-B)
    test("4.5 — phase-stepper visible after launch", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await page.goto("/admin");
      await expect(page.locator("main")).toBeVisible({ timeout: 15000 });
      // Phase-stepper appears when a season is active and launched.
      await expect(page.getByTestId("phase-stepper")).toBeVisible();
    });

    // Story 4.6: Enrollment phase — only season-open SMS button shown (NEW-4.6-A)
    test("4.6 — enrollment phase shows only season-open SMS button", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await page.goto("/admin");
      await expect(page.locator("main")).toBeVisible({ timeout: 15000 });

      // Correct phase signal: season-open button is visible
      await expect(page.getByTestId("send-season-open-button")).toBeVisible();

      // Wrong-phase SMS buttons are not in the DOM in Enrollment phase
      await expect(page.getByTestId("send-confirm-nudge-button")).not.toBeVisible();
      await expect(page.getByTestId("send-assignment-button")).not.toBeVisible();
      await expect(page.getByTestId("send-receipt-nudge-button")).not.toBeVisible();
    });

    // Story 4.7 AC: advance-blocked-hint not shown in Enrollment phase
    test("4.7 — advance not blocked in enrollment phase", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await page.goto("/admin");
      // Wait for Suspense to resolve — advance-button must be visible first.
      await expect(page.getByTestId("advance-button")).toBeVisible();
      await expect(page.getByTestId("advance-blocked-hint")).not.toBeVisible();
    });

    // Story 5.3: Season-open SMS — also asserts 4.4-A count (active users)
    // FIX-3: The unified admin page is phase-gated (Story 4.6). Only
    // sms-count-active-users is rendered in Enrollment phase. The other three
    // count spans (unnotified-senders, unconfirmed-enrolled, no-response) are
    // absent from the DOM until their respective phases.
    test("5.3 — admin triggers season-open SMS", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      // Story 4.4-A: active user count visible in Enrollment phase
      await page.goto("/admin");
      await app.expectSmsCountVisible("sms-count-active-users");
      // 3 active participants registered in test setup
      await app.expectSmsCount("sms-count-active-users", "3");
      await app.triggerSms("season-open");
      await app.expectSmsReport();
      await expect(page.getByTestId("sms-sent-confirmation")).toBeVisible();
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
      await expect(page.getByTestId("season-theme")).toBeVisible();
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
      await app.enrollInSeason(CITY_KYIV, BRANCH_1);
      await app.expectEnrolled();
    });

    // Story 2.1 AC: can update Nova Poshta branch during enrollment
    test("2.1 — participant can set branch during enrollment", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.B);
      await app.goHome();
      await app.enrollInSeason(CITY_LVIV, BRANCH_10);
      await app.expectEnrolled();
    });

    // Setup: enroll participant C
    test("setup — participant C enrolls", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.C);
      await app.goHome();
      await app.enrollInSeason(CITY_KYIV, BRANCH_5);
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
    // Phase advancement: enrollment → preparation
    // In test mode, advancement bypasses deadline gates.
    test("phase — advance enrollment → preparation", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.advanceSeason();
      await app.goToDashboard();
      await app.expectDashboardContent(/preparation|підготовка/i);
    });

    // Story 4.6: Preparation phase — only confirm-nudge SMS button shown (NEW-4.6-B)
    test("4.6 — preparation phase shows only confirm-nudge SMS button", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await page.goto("/admin");
      await expect(page.locator("main")).toBeVisible({ timeout: 15000 });

      // Correct phase signal: confirm-nudge button is visible
      await expect(page.getByTestId("send-confirm-nudge-button")).toBeVisible();

      // Wrong-phase SMS buttons are not in the DOM in Preparation phase
      await expect(page.getByTestId("send-season-open-button")).not.toBeVisible();
      await expect(page.getByTestId("send-assignment-button")).not.toBeVisible();
      await expect(page.getByTestId("send-receipt-nudge-button")).not.toBeVisible();
    });

    // Architecture Home Screen: creating phase, enrolled not confirmed
    test("home screen — creation period message shown", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.A);
      await app.goHome();
      // "Create your mail. Confirm ready by [deadline]." + countdown
      await app.expectHomeContent(/create|створ/i);
    });

    // Story 4.4-B: confirm-nudge count shows unconfirmed enrolled count
    // Placement: Preparation phase — 3 enrolled, none confirmed yet.
    test("4.4 — confirm-nudge count shows unconfirmed enrolled participants", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await page.goto("/admin");
      await app.expectSmsCountVisible("sms-count-unconfirmed-enrolled");
      // All 3 enrolled participants have not yet confirmed ready.
      await app.expectSmsCount("sms-count-unconfirmed-enrolled", "3");
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
      await expect(page.getByTestId("season-deadline")).toBeVisible();
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
    test("phase — advance preparation → assignment", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.advanceSeason();
      await app.goToDashboard();
      await app.expectDashboardContent(/assignment|розподіл/i);
    });

    // Story 4.6: Assignment phase — no SMS buttons shown (NEW-4.6-C)
    test("4.6 — assignment phase shows no SMS buttons", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await page.goto("/admin");
      await expect(page.locator("main")).toBeVisible({ timeout: 15000 });

      // Phase signal: generate-button is the canonical Assignment-phase indicator
      await expect(page.getByTestId("generate-button")).toBeVisible();

      // No SMS buttons in Assignment phase
      await expect(page.getByTestId("send-season-open-button")).not.toBeVisible();
      await expect(page.getByTestId("send-confirm-nudge-button")).not.toBeVisible();
      await expect(page.getByTestId("send-assignment-button")).not.toBeVisible();
      await expect(page.getByTestId("send-receipt-nudge-button")).not.toBeVisible();
    });

    // Architecture Home Screen: assigning phase
    test("home screen — participant sees 'preparing' during assignment", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.A);
      await app.goHome();
      await app.expectHomeContent(/розподіл|хвилин|надіслати/i);
    });

    // Story 3.1 AC: organizer sees confirmed count before generating
    test("3.1 — admin sees confirmed count before generating", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await page.goto("/admin");
      // Confirmed count should be visible (3 participants)
      await expect(page.getByTestId("confirmed-count")).toBeVisible();
    });

    // Story 4.7-A: advance button disabled in Assignment phase before generate
    // Season just entered Assignment phase — assignments table is empty for this season.
    test("4.7 — advance blocked before assignments are generated", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await page.goto("/admin");
      // Wait for Suspense to resolve — advance-button must be present.
      await expect(page.getByTestId("advance-button")).toBeVisible();
      await app.expectAdvanceBlocked();
    });

    // Story 3.1: Generate assignments
    // Story 4.7-B: advance button enabled after generate (assert inside)
    test("3.1 — admin generates assignments", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.generateAssignments();
      await app.expectCycleVisualization();
      // Story 4.7-B: advance is now unblocked since assignments exist in DB.
      await page.goto("/admin");
      await expect(page.getByTestId("advance-button")).toBeVisible();
      await app.expectAdvanceEnabled();
    });

    // Story 3.3 AC: assignments not visible to participants before delivery
    test("3.3 — participant cannot see assignment before delivery", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.A);
      await app.goHome();
      // Still shows "preparing" — not the assignment details
      await expect(app.page.getByTestId("recipient-name")).not.toBeVisible();
    });

    // Story 3.3: Swap/override UI available
    // Story 4.8-A: swap form displays select dropdowns (not text inputs)
    test("3.3 — swap UI available to admin", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await page.goto("/admin");
      await expect(page.getByTestId("override-available")).toBeVisible();
      // Story 4.8-A: sender-a-input and sender-b-input are <select> elements rendered inside override-available.
      await expect(page.getByTestId("sender-a-input")).toBeVisible();
      await expect(page.getByTestId("sender-b-input")).toBeVisible();
    });

    // Story 3.3: Swap two assignments and verify cycle remains valid
    test("3.3 — admin swaps two assignments", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.swapAssignment(NAMES.A, NAMES.B);
      await app.expectCycleVisualization();
    });
  });

  // ════════════════════════════════════════════
  // BLOCK 6: Delivery (Stories 2.3, 2.4, 5.2)
  // ════════════════════════════════════════════

  test.describe("Stories 2.3–2.4: Delivery & Receipt", () => {
    // Phase advancement: assignment → delivery
    test("phase — advance assignment → delivery", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.advanceSeason();
      await app.goToDashboard();
      await app.expectDashboardContent(/delivery|відправлення|доставка/i);
    });

    // Story 4.6: Delivery phase — assignment + receipt-nudge SMS buttons shown, not others (NEW-4.6-D)
    test("4.6 — delivery phase shows assignment and receipt-nudge SMS buttons only", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await page.goto("/admin");
      await expect(page.locator("main")).toBeVisible({ timeout: 15000 });

      // Correct phase signals: both Delivery SMS buttons are visible
      await expect(page.getByTestId("send-assignment-button")).toBeVisible();
      await expect(page.getByTestId("send-receipt-nudge-button")).toBeVisible();

      // Wrong-phase SMS buttons are not in the DOM in Delivery phase
      await expect(page.getByTestId("send-season-open-button")).not.toBeVisible();
      await expect(page.getByTestId("send-confirm-nudge-button")).not.toBeVisible();
    });

    // Story 5.1: Assignment SMS
    // Story 4.4-C: unnotified sender count visible and drops to 0 after sending
    // Placement: Delivery phase — send-assignment-button and sms-count-unnotified-senders
    // are only rendered in Delivery phase (per Story 4.6 phase-gating).
    test("5.1 — admin triggers assignment SMS", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      // Story 4.4-C before send: count > 0 (3 senders not yet notified).
      await page.goto("/admin");
      await app.expectSmsCountVisible("sms-count-unnotified-senders");
      await expect(
        page.getByTestId("sms-count-unnotified-senders"),
      ).not.toContainText("0");
      // Send the assignment SMS.
      await page.getByTestId("send-assignment-button").click();
      await app.expectSmsReport();
      // Story 4.4-C after send: notified_at set on all 3 assignment rows → count drops to 0.
      await app.expectSmsCount("sms-count-unnotified-senders", "0");
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

    // Story 5.2: Receipt nudge SMS
    test("5.2 — admin triggers receipt-nudge SMS", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.triggerSms("receipt-nudge");
      await app.expectSmsReport();
    });

    // Architecture Home Screen: receiving phase
    test("home screen — delivery phase prompt shown", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.A);
      await app.goHome();
      await app.expectHomeContent(/відправ|отримав/i);
    });

    // Story 2.4: Confirm receipt — received
    test("2.4 — participant confirms receipt (received)", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.A);
      await app.goHome();
      await app.confirmReceipt(true);
      await expect(page.getByTestId("receipt-thanks")).toBeVisible();
    });

    // Story 2.4: Confirm receipt — not received, with note
    test("2.4 — participant reports not received with note", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.B);
      await app.goHome();
      await app.confirmReceipt(false, "Пошта не надійшла");
      await expect(page.getByTestId("receipt-thanks")).toBeVisible();
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
    test("phase — advance delivery → complete", async ({ page }) => {
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
  // BLOCK 8: Cancel Season
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

    test("cancel — back button dismisses confirmation", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await page.goto("/admin");
      await page.getByTestId("cancel-button").click();
      await expect(page.getByTestId("cancel-confirmation")).toBeVisible();
      await page.getByTestId("cancel-back-button").click();
      await expect(page.getByTestId("cancel-confirmation")).not.toBeVisible();
      await expect(page.getByTestId("cancel-button")).toBeVisible();
    });

    test("cancel — admin cancels the season", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.cancelSeason();
      await app.goToDashboard();
      // Cancelled phase renders is_terminal=true. The "Новий сезон" link appears
      // in terminal state. Neither "cancelled", "скасовано", "no active", nor "create"
      // appear as standalone text in this view. (FIX-2)
      await app.expectDashboardContent(/новий сезон/i);
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

// ════════════════════════════════════════════════════
// INDEPENDENT BLOCK: Account Management (Epic 6)
// Story: 6.1
// Creates its own participant — no dependency on season lifecycle.
// ════════════════════════════════════════════════════

test.describe.serial("Account Management", () => {
  const DEACTIVATE_PHONE = "+380670000099";
  const DEACTIVATE_NAME = "Деактивований Учасник";

  // Setup: admin generates a code, then participant self-registers
  test("setup — generate invite code for deactivation target", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(ADMIN_PHONE);
    CODES.DEACTIVATE = await app.generateInviteCode();
    expect(CODES.DEACTIVATE.length).toBeGreaterThan(0);
  });

  test("setup — deactivation target self-registers", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.selfRegister(DEACTIVATE_PHONE, CODES.DEACTIVATE, DEACTIVATE_NAME);
    // Registration completes at /onboarding — participant exists.
    await expect(page).toHaveURL(/\/onboarding/);
  });

  // Verify the participant appears in the admin list before deactivation.
  test("setup — deactivation target appears in participant list", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(ADMIN_PHONE);
    await page.goto("/admin");
    await app.expectParticipantInList(DEACTIVATE_NAME);
  });

  // Story 6.1: Deactivate participant
  test("6.1 — admin deactivates a participant", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(ADMIN_PHONE);
    await app.deactivateParticipant(DEACTIVATE_NAME);
    await expect(page.getByTestId("inactive-status")).toBeVisible();
  });

  // Story 6.1 AC: deactivated account cannot sign in
  test("6.1 — deactivated participant cannot sign in", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.attemptLogin(DEACTIVATE_PHONE);
    // Should remain on login — auth rejected
    await expect(page).toHaveURL(/\/login/);
  });
});

// ════════════════════════════════════════════════════
// INDEPENDENT BLOCK: Session Management
// Verifies logout functionality (clears session, redirects to login).
// Uses PHONES.A (created in Epic 1) and ADMIN_PHONE (seeded).
// Only tests session management — no dependency on season lifecycle.
// ════════════════════════════════════════════════════

test.describe.serial("Session Management", () => {
  test("logout — participant logs out and is redirected to login", async ({
    page,
  }) => {
    const app = new MailClubPage(page);
    // Login as participant
    await app.login(PHONES.A);
    await app.goHome();
    // Logout
    await app.logout();
    // Should be at / which redirects to /login (no session)
    await page.waitForURL(/\/login/);
  });

  test("logout — admin logs out and is redirected to login", async ({
    page,
  }) => {
    const app = new MailClubPage(page);
    // Login as admin
    await app.login(ADMIN_PHONE);
    await app.goToDashboard();
    // Logout
    await app.logout();
    // Should be at / which redirects to /login (no session)
    await page.waitForURL(/\/login/);
  });
});
