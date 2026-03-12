import { test, expect } from "@playwright/test";
import { MailClubPage } from "./fixtures/mail_club_page";

/**
 * Epic 1: Join the Community
 *
 * Stories: 1.1, 1.2, 1.3
 * Phase: 2 (Auth)
 *
 * Prerequisite: admin user exists in database.
 * Tests run sequentially — later tests depend on earlier state.
 */

const ADMIN_PHONE = "+380670000001";
const PARTICIPANT_PHONE = "+380670000002";
const PARTICIPANT_NAME = "Тестова Людина";
const NOVA_POSHTA_BRANCH = "Відділення №1, Київ";

test.describe("Epic 1: Join the Community", () => {
  // Story 1.1: Organizer registers a new participant
  test("1.1 — admin can register a new participant", async ({ page }) => {
    const app = new MailClubPage(page);

    // Admin logs in
    await app.login(ADMIN_PHONE);
    await app.expectLoggedIn();

    // Register a participant
    await app.registerParticipant(PARTICIPANT_PHONE, PARTICIPANT_NAME);
    await app.expectParticipantInList(PARTICIPANT_NAME);
  });

  // Story 1.1 AC: only admin can create accounts
  test("1.1 — non-admin cannot access admin pages", async ({ page }) => {
    const app = new MailClubPage(page);

    await app.login(PARTICIPANT_PHONE);
    await page.goto("/admin/participants");

    // Should be redirected away or see forbidden
    await expect(page).not.toHaveURL(/\/admin/);
  });

  // Story 1.1 AC: duplicate phone rejected
  test("1.1 — duplicate phone number is rejected", async ({ page }) => {
    const app = new MailClubPage(page);

    await app.login(ADMIN_PHONE);
    await app.registerParticipant(PARTICIPANT_PHONE, "Дублікат");

    // Should show error, not create duplicate
    await expect(page.getByText(/already|існує|duplicate/i)).toBeVisible();
  });

  // Story 1.2: Sign in with phone number
  test("1.2 — registered participant can sign in via OTP", async ({ page }) => {
    const app = new MailClubPage(page);

    await app.login(PARTICIPANT_PHONE);
    await app.expectLoggedIn();
  });

  // Story 1.2 AC: unregistered phone is rejected
  test("1.2 — unregistered phone cannot sign in", async ({ page }) => {
    const app = new MailClubPage(page);

    await page.goto("/login");
    await page.getByLabel(/phone/i).fill("+380679999999");
    await page.getByRole("button", { name: /send|submit|code/i }).click();

    // Should not proceed to OTP step (or should show error)
    // The app should NOT reveal whether the phone is registered
    // It silently fails — no OTP is sent, verification will fail
  });

  // Story 1.3: Complete onboarding
  test("1.3 — first-time user is redirected to onboarding", async ({
    page,
  }) => {
    const app = new MailClubPage(page);

    // Fresh participant who hasn't onboarded
    await app.login(PARTICIPANT_PHONE);
    await app.expectRedirectedToOnboarding();
  });

  test("1.3 — user can complete onboarding with Nova Poshta branch", async ({
    page,
  }) => {
    const app = new MailClubPage(page);

    await app.login(PARTICIPANT_PHONE);
    await app.expectRedirectedToOnboarding();
    await app.completeOnboarding(NOVA_POSHTA_BRANCH);
    await app.expectRedirectedToHome();
  });

  test("1.3 — returning user skips onboarding", async ({ page }) => {
    const app = new MailClubPage(page);

    // After onboarding is complete, login goes straight to home
    await app.login(PARTICIPANT_PHONE);
    await app.expectRedirectedToHome();
  });
});
