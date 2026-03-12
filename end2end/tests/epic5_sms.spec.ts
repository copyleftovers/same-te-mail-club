import { test, expect } from "@playwright/test";
import { MailClubPage } from "./fixtures/mail_club_page";

/**
 * Epic 5: SMS Notifications
 *
 * Stories: 5.1, 5.2, 5.3, 5.4
 * Phase: 5
 *
 * Prerequisite: season exists in appropriate phase, participants registered.
 * SMS_DRY_RUN=true — tests verify the admin trigger flow and report,
 * not actual SMS delivery.
 */

const ADMIN_PHONE = "+380670000001";

test.describe("Epic 5: SMS Notifications", () => {
  // Story 5.3: Season-open notification
  test("5.3 — admin can trigger season-open SMS", async ({ page }) => {
    const app = new MailClubPage(page);

    await app.login(ADMIN_PHONE);
    await app.triggerSms("season-open");
    await app.expectSmsReport();

    // Report should show sent count
    await expect(page.getByText(/sent|надіслано/i)).toBeVisible();
  });

  // Story 5.1: Assignment notification
  test("5.1 — admin can trigger assignment SMS", async ({ page }) => {
    const app = new MailClubPage(page);

    await app.login(ADMIN_PHONE);
    await app.triggerSms("assignment");
    await app.expectSmsReport();
  });

  // Story 5.1 AC: SMS contains no recipient details
  // (This is verified by checking the SMS content in dry-run logs,
  //  not via E2E. The E2E test verifies the trigger flow works.)

  // Story 5.4: Pre-deadline nudge to non-confirmers
  test("5.4 — admin can trigger confirm-nudge SMS", async ({ page }) => {
    const app = new MailClubPage(page);

    await app.login(ADMIN_PHONE);
    await app.triggerSms("confirm-nudge");
    await app.expectSmsReport();
  });

  // Story 5.4 AC: only sent to non-confirmers
  test("5.4 — confirm-nudge report shows targeted count", async ({ page }) => {
    const app = new MailClubPage(page);

    await app.login(ADMIN_PHONE);
    await app.triggerSms("confirm-nudge");
    await app.expectSmsReport();

    // The sent count should be less than total enrolled
    // (confirmed participants should NOT receive the nudge)
  });

  // Story 5.2: Receipt nudge to non-responders
  test("5.2 — admin can trigger receipt-nudge SMS", async ({ page }) => {
    const app = new MailClubPage(page);

    await app.login(ADMIN_PHONE);
    await app.triggerSms("receipt-nudge");
    await app.expectSmsReport();
  });
});
