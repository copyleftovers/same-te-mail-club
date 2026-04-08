import { test, expect } from "@playwright/test";
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

const NAMES = {
  A: "Тестова Людина А",
  B: "Тестова Людина Б",
  C: "Тестова Людина В",
};

const CITY_KYIV = "Київ";
const CITY_BROVARY = "Бровари";
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
      await app.registerParticipant(PHONES.B, NAMES.B);
      await app.registerParticipant(PHONES.C, NAMES.C);
    });

    // Story 1.1 AC: duplicate phone rejected
    test("1.1 — duplicate phone number is rejected", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.registerParticipant(PHONES.A, "Дублікат");
      await expect(page.getByTestId("action-error")).toContainText(/already exists|існує/i);
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

    // Setup: onboard remaining participants (B and C)
    test("setup — onboard participants B and C", async ({ page }) => {
      const app = new MailClubPage(page);

      await app.login(PHONES.B);
      await app.expectRedirectedToOnboarding();
      await app.completeOnboarding(CITY_KYIV, BRANCH_1);
      await app.expectRedirectedToHome();

      // Fresh browser context for C — need new page
    });

    test("setup — onboard participant C", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.C);
      await app.expectRedirectedToOnboarding();
      await app.completeOnboarding(CITY_KYIV, BRANCH_1);
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
    // When an active season exists, the create form is hidden — the page shows
    // the management panel instead. No second season can be created.
    test("4.1 — second active season is rejected", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await page.goto("/admin/season");
      // Create form should NOT be visible (season already exists)
      await expect(page.getByTestId("create-season-button")).not.toBeVisible();
      // Management panel should be visible instead
      await expect(page.getByTestId("launch-button")).toBeVisible();
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

    // Architecture Home Screen: creating phase, enrolled not confirmed
    test("home screen — creation period message shown", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.A);
      await app.goHome();
      // "Create your mail. Confirm ready by [deadline]." + countdown
      await app.expectHomeContent(/create|створ/i);
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
      await page.goto("/admin/assignments");
      // Confirmed count should be visible (3 participants)
      await expect(page.getByTestId("confirmed-count")).toBeVisible();
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
      await expect(page.getByTestId("override-available")).toBeVisible();
    });

    // Story 3.3: Release assignments
    test("3.3 — admin releases assignments", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await page.goto("/admin/assignments");
      await app.releaseAssignments();
      await expect(page.getByTestId("released-status")).toBeVisible();
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
    // Phase advancement: assignment → delivery
    test("phase — advance assignment → delivery", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.advanceSeason();
      await app.goToDashboard();
      await app.expectDashboardContent(/delivery|відправлення|доставка/i);
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
      // Reveal envelope before counting
      await app.revealAssignment();
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

// ════════════════════════════════════════════════════
// INDEPENDENT BLOCK: Account Management (Epic 6)
// Story: 6.1
// Creates its own participant — no dependency on season lifecycle.
// ════════════════════════════════════════════════════

test.describe.serial("Account Management", () => {
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
