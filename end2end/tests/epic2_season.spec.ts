import { test, expect } from "@playwright/test";
import { MailClubPage } from "./fixtures/mail_club_page";

/**
 * Epic 2: Participate in a Season
 *
 * Stories: 2.1, 2.2, 2.3, 2.4
 * Phases: 3 (enrollment/confirm), 5 (assignment view/receipt)
 *
 * Prerequisite: admin and participants exist, season created and in signup phase.
 * Depends on: Epic 1 (users exist), Epic 4 (season created).
 */

const ADMIN_PHONE = "+380670000001";
const PARTICIPANT_A = "+380670000002";
const PARTICIPANT_B = "+380670000003";
const PARTICIPANT_C = "+380670000004";
const BRANCH = "Відділення №1, Київ";

test.describe("Epic 2: Participate in a Season", () => {
  // Story 2.1: Enroll in an upcoming season
  test("2.1 — participant can enroll during signup phase", async ({ page }) => {
    const app = new MailClubPage(page);

    await app.login(PARTICIPANT_A);
    await page.goto("/");

    // Should see enroll button when season is in signup phase
    await expect(app.page.getByTestId("enroll-button")).toBeVisible();
    await app.enrollInSeason(BRANCH);
    await app.expectEnrolled();
  });

  // Story 2.1 AC: enrollment shows content guidelines
  test("2.1 — content guidelines are visible during enrollment", async ({
    page,
  }) => {
    const app = new MailClubPage(page);

    await app.login(PARTICIPANT_B);
    await page.goto("/");

    // Content guidelines should be visible before/during enrollment
    await expect(page.getByText(/self-expression|самовираження/i)).toBeVisible();
  });

  // Story 2.1 AC: can update Nova Poshta branch during enrollment
  test("2.1 — participant can update branch during enrollment", async ({
    page,
  }) => {
    const app = new MailClubPage(page);

    await app.login(PARTICIPANT_B);
    await page.goto("/");

    const newBranch = "Відділення №5, Київ";
    await app.enrollInSeason(newBranch);
    await app.expectEnrolled();
  });

  // Story 2.2: Confirm mail is ready
  test("2.2 — enrolled participant can confirm ready", async ({ page }) => {
    const app = new MailClubPage(page);

    // Season must be in creating or confirming phase
    await app.login(PARTICIPANT_A);
    await page.goto("/");

    await expect(app.page.getByTestId("confirm-ready-button")).toBeVisible();
    await app.confirmReady();
    await app.expectConfirmed();
  });

  // Story 2.2 AC: confirmation is irreversible
  test("2.2 — confirmed participant cannot un-confirm", async ({ page }) => {
    const app = new MailClubPage(page);

    await app.login(PARTICIPANT_A);
    await page.goto("/");

    // Confirm button should not be visible after confirmation
    await expect(
      app.page.getByTestId("confirm-ready-button"),
    ).not.toBeVisible();
  });

  // Story 2.2 AC: deadline countdown is visible
  test("2.2 — ready-confirm deadline is visible with countdown", async ({
    page,
  }) => {
    const app = new MailClubPage(page);

    await app.login(PARTICIPANT_B);
    await page.goto("/");

    await expect(page.getByText(/deadline|дедлайн|залишилось/i)).toBeVisible();
  });

  // Story 2.3: Receive assignment
  test("2.3 — assigned participant sees recipient details", async ({
    page,
  }) => {
    const app = new MailClubPage(page);

    // Season must be in sending phase with released assignments
    await app.login(PARTICIPANT_A);
    await page.goto("/");

    await app.expectAssignmentVisible();
    const assignment = await app.getAssignment();
    expect(assignment.name).toBeTruthy();
    expect(assignment.phone).toMatch(/\+380/);
    expect(assignment.branch).toBeTruthy();
  });

  // Story 2.3 AC: participant sees ONLY their own recipient
  test("2.3 — participant cannot see other cohort members", async ({
    page,
  }) => {
    const app = new MailClubPage(page);

    await app.login(PARTICIPANT_A);
    await page.goto("/");

    // Should see exactly one recipient, no list of cohort members
    const recipientNames = await app.page
      .getByTestId("recipient-name")
      .count();
    expect(recipientNames).toBe(1);
  });

  // Story 2.4: Confirm mail received
  test("2.4 — participant can confirm receipt (received)", async ({ page }) => {
    const app = new MailClubPage(page);

    // Season must be in receiving phase
    await app.login(PARTICIPANT_A);
    await page.goto("/");

    await app.confirmReceipt(true);
    await expect(page.getByText(/дякуємо|thanks|confirmed/i)).toBeVisible();
  });

  // Story 2.4 AC: not-received triggers organizer notification
  test("2.4 — participant can report not received with note", async ({
    page,
  }) => {
    const app = new MailClubPage(page);

    await app.login(PARTICIPANT_B);
    await page.goto("/");

    await app.confirmReceipt(false, "Пошта не надійшла");
    // Should see confirmation of the report
    await expect(page.getByText(/reported|повідомлено/i)).toBeVisible();
  });
});
