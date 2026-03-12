import { test, expect } from "@playwright/test";
import { MailClubPage } from "./fixtures/mail_club_page";

/**
 * Epic 4: Season Management (Organizer)
 *
 * Stories: 4.1, 4.2
 * Phase: 3
 *
 * Prerequisite: admin user exists.
 */

const ADMIN_PHONE = "+380670000001";

test.describe("Epic 4: Season Management", () => {
  // Story 4.1: Create a new season
  test("4.1 — admin can create a season with deadlines", async ({ page }) => {
    const app = new MailClubPage(page);

    await app.login(ADMIN_PHONE);

    // Future dates for deadlines
    const now = new Date();
    const signup = new Date(now.getTime() + 5 * 24 * 60 * 60 * 1000);
    const confirm = new Date(now.getTime() + 14 * 24 * 60 * 60 * 1000);

    await app.createSeason(
      signup.toISOString().slice(0, 16),
      confirm.toISOString().slice(0, 16),
      "Тестова тема",
    );

    // Should see the created season on the dashboard
    await app.goToDashboard();
    await expect(page.getByText(/signup|реєстрація/i)).toBeVisible();
  });

  // Story 4.1 AC: only admin can create seasons
  test("4.1 — non-admin cannot create seasons", async ({ page }) => {
    await page.goto("/admin/season");
    // Not logged in — should redirect to login
    await expect(page).toHaveURL(/\/login/);
  });

  // Story 4.1 AC: cannot create second active season
  test("4.1 — creating a second active season is rejected", async ({
    page,
  }) => {
    const app = new MailClubPage(page);

    await app.login(ADMIN_PHONE);

    const now = new Date();
    const signup = new Date(now.getTime() + 5 * 24 * 60 * 60 * 1000);
    const confirm = new Date(now.getTime() + 14 * 24 * 60 * 60 * 1000);

    await app.createSeason(
      signup.toISOString().slice(0, 16),
      confirm.toISOString().slice(0, 16),
    );

    // Should show error about existing active season
    await expect(
      page.getByText(/already|active|існує|активний/i),
    ).toBeVisible();
  });

  // Story 4.2: Launch — in this architecture, creation IS launch (signup phase)
  // The season is visible to participants immediately after creation.
  test("4.2 — created season is visible to participants", async ({ page }) => {
    const app = new MailClubPage(page);

    // Login as a participant
    await app.login("+380670000002");
    await page.goto("/");

    // Should see the season (enroll option)
    await expect(page.getByTestId("enroll-button")).toBeVisible();
  });

  // Admin can advance season phases
  test("admin can advance season from signup to creating", async ({
    page,
  }) => {
    const app = new MailClubPage(page);

    await app.login(ADMIN_PHONE);
    await app.advanceSeason();

    await app.goToDashboard();
    await expect(page.getByText(/creating|створення/i)).toBeVisible();
  });

  // Admin dashboard shows counts
  test("dashboard shows enrolled and confirmed counts", async ({ page }) => {
    const app = new MailClubPage(page);

    await app.login(ADMIN_PHONE);
    await app.goToDashboard();

    // Dashboard should display enrollment and confirmation counts
    await expect(page.getByText(/enrolled|зареєстровано/i)).toBeVisible();
  });
});
