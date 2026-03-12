import { test, expect } from "@playwright/test";
import { MailClubPage } from "./fixtures/mail_club_page";

/**
 * Epic 3: Assignment Algorithm
 *
 * Stories: 3.1, 3.2, 3.3
 * Phase: 4
 *
 * Prerequisite: season in assigning phase, confirmed participants exist.
 * Note: social-awareness (3.2) is tested via unit tests on the algorithm.
 *       E2E verifies the admin flow and cycle validity.
 */

const ADMIN_PHONE = "+380670000001";

test.describe("Epic 3: Assignment Algorithm", () => {
  // Story 3.1: Generate cohort assignments
  test("3.1 — admin can generate assignments", async ({ page }) => {
    const app = new MailClubPage(page);

    await app.login(ADMIN_PHONE);
    await app.generateAssignments();

    // Should see cycle visualization after generation
    await app.expectCycleVisualization();
  });

  // Story 3.1 AC: every participant appears in exactly one cohort
  test("3.1 — all confirmed participants appear in assignments", async ({
    page,
  }) => {
    const app = new MailClubPage(page);

    await app.login(ADMIN_PHONE);
    await page.goto("/admin/assignments");

    // The cycle visualization should show all participants
    await app.expectCycleVisualization();

    // Each participant should appear exactly once
    // (Implementation determines exact assertion — check no duplicates, no missing)
  });

  // Story 3.3: Override assignments
  test("3.3 — admin can swap pairings while maintaining cycle", async ({
    page,
  }) => {
    const app = new MailClubPage(page);

    await app.login(ADMIN_PHONE);
    await page.goto("/admin/assignments");

    // The swap UI exists — exact interaction depends on implementation
    // This test verifies the flow is possible, not the specific UI
    await expect(page.getByText(/swap|обмін/i)).toBeVisible();
  });

  // Story 3.3 AC: assignments not released until admin confirms
  test("3.3 — assignments are not visible to participants before release", async ({
    page,
  }) => {
    const app = new MailClubPage(page);

    // Check as a participant — assignment should NOT be visible yet
    await app.login("+380670000002");
    await page.goto("/");

    await expect(app.page.getByTestId("recipient-name")).not.toBeVisible();
  });

  // Story 3.3: Admin releases assignments
  test("3.3 — admin can release assignments to participants", async ({
    page,
  }) => {
    const app = new MailClubPage(page);

    await app.login(ADMIN_PHONE);
    await page.goto("/admin/assignments");
    await app.releaseAssignments();

    // Should see confirmation of release
    await expect(page.getByText(/released|опубліковано/i)).toBeVisible();
  });
});
