import { test, expect } from "@playwright/test";
import { MailClubPage } from "./fixtures/mail_club_page";

/**
 * Epic 6: Account Management (Organizer)
 *
 * Story: 6.1
 * Phase: 6
 *
 * Prerequisite: admin and participant accounts exist.
 */

const ADMIN_PHONE = "+380670000001";
const TARGET_PHONE = "+380670000099";
const TARGET_NAME = "Деактивований Учасник";

test.describe("Epic 6: Account Management", () => {
  // Setup: register the participant to be deactivated
  test("setup — register participant for deactivation test", async ({
    page,
  }) => {
    const app = new MailClubPage(page);

    await app.login(ADMIN_PHONE);
    await app.registerParticipant(TARGET_PHONE, TARGET_NAME);
    await app.expectParticipantInList(TARGET_NAME);
  });

  // Story 6.1: Deactivate a participant's account
  test("6.1 — admin can deactivate a participant", async ({ page }) => {
    const app = new MailClubPage(page);

    await app.login(ADMIN_PHONE);
    await app.deactivateParticipant(TARGET_NAME);

    // Participant should be marked as inactive in the list
    await expect(page.getByText(/inactive|деактивовано/i)).toBeVisible();
  });

  // Story 6.1 AC: deactivated account cannot sign in
  test("6.1 — deactivated participant cannot sign in", async ({ page }) => {
    const app = new MailClubPage(page);

    await page.goto("/login");
    await page.getByLabel(/phone/i).fill(TARGET_PHONE);
    await page.getByRole("button", { name: /send|submit|code/i }).click();

    // OTP step — code entry should either not appear or verification should fail
    // The app rejects OTP requests for inactive accounts
    await page.getByLabel(/code/i).fill("000000");
    await page.getByRole("button", { name: /verify|submit|sign/i }).click();

    // Should NOT be logged in
    await expect(page).toHaveURL(/\/login/);
  });

  // Story 6.1 AC: deactivated participants excluded from season-open SMS
  // (Verified via unit test on the SMS recipient query, not E2E)
});
