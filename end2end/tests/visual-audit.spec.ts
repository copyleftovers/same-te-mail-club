import { type Page } from "@playwright/test";
import { test, expect } from "./fixtures/cached-context";
import { MailClubPage } from "./fixtures/mail_club_page";
import * as fs from "fs";

/**
 * Visual Audit — screenshot harvester for every meaningful app state.
 *
 * One serial block. State builds progressively (same pattern as mail_club.spec.ts).
 * Each test navigates to a state, captures mobile + desktop screenshots, moves on.
 *
 * No assertions beyond what is needed to confirm the page has reached the target
 * state before the camera fires. This spec documents the UI; mail_club.spec.ts
 * verifies correctness.
 *
 * Screenshot paths resolve relative to end2end/ (the Playwright working directory):
 *   screenshots/mobile/NN-page-state.png
 *   screenshots/desktop/NN-page-state.png
 *
 * Env (same as main suite):
 *   SAMETE_TEST_MODE=true   — fixed OTP "000000", deadline gates bypassed
 *   SAMETE_SMS_DRY_RUN=true — SMS logged, not sent
 */

// ── Phone constants ────────────────────────────────────────────────────────────
// Separate phones from the main suite so the two suites can run independently.

const ADMIN_PHONE = "+380670000001";

const AUDIT_PHONES = {
  A: "+380680000002",
  B: "+380680000003",
  C: "+380680000004",
  NEW: "+380680000005",
};

const AUDIT_NAMES = {
  A: "Аудит Учасник А",
  B: "Аудит Учасник Б",
  C: "Аудит Учасник В",
};

// ── Deadline helpers ───────────────────────────────────────────────────────────

function futureDatetime(daysFromNow: number): string {
  const d = new Date(Date.now() + daysFromNow * 24 * 60 * 60 * 1000);
  return d.toISOString().slice(0, 16);
}

const SIGNUP_DEADLINE = futureDatetime(7);
const CONFIRM_DEADLINE = futureDatetime(21);
const SEASON_THEME = "Аудит — Перший сезон";

// ── Invite codes captured during the run ──────────────────────────────────────

const AUDIT_CODES = { A: "", B: "", C: "", NEW: "" };

// ── Sequence counter shared across all tests in the serial block ───────────────
// Module-level mutable object gives all tests a single incrementing counter,
// producing globally ordered filenames without any inter-test coordination.

const SEQ = { n: 1 };

// ── Screenshot helper ──────────────────────────────────────────────────────────

const MOBILE_VIEWPORT = { width: 375, height: 812 } as const;
const DESKTOP_VIEWPORT = { width: 1280, height: 800 } as const;
const LAYOUT_REFLOW_MS = 150;

async function captureState(page: Page, name: string): Promise<void> {
  const paddedIndex = String(SEQ.n).padStart(2, "0");
  SEQ.n += 1;

  await page.setViewportSize(MOBILE_VIEWPORT);
  await page.waitForTimeout(LAYOUT_REFLOW_MS);
  await page.screenshot({
    path: `screenshots/mobile/${paddedIndex}-${name}.png`,
    fullPage: true,
  });

  await page.setViewportSize(DESKTOP_VIEWPORT);
  await page.waitForTimeout(LAYOUT_REFLOW_MS);
  await page.screenshot({
    path: `screenshots/desktop/${paddedIndex}-${name}.png`,
    fullPage: true,
  });
}

// ── Directory creation ─────────────────────────────────────────────────────────

test.beforeAll(() => {
  fs.mkdirSync("screenshots/mobile", { recursive: true });
  fs.mkdirSync("screenshots/desktop", { recursive: true });
});

// ── Visual Audit ──────────────────────────────────────────────────────────────

test.describe.serial("Visual Audit", () => {

  // ── Login flow (admin) ───────────────────────────────────────────────────────

  test("capture login — phone input", async ({ page }) => {
    await page.goto("/login");
    await expect(page.getByTestId("send-otp-button")).toBeEnabled();
    await captureState(page, "login-phone-input");
  });

  test("capture login — otp input", async ({ page }) => {
    const app = new MailClubPage(page);
    await page.goto("/login");
    await expect(page.getByTestId("send-otp-button")).toBeEnabled();
    await page.getByTestId("phone-input").fill(ADMIN_PHONE);
    await app.clickAndWaitForResponse(
      page.getByTestId("send-otp-button"),
      "request_otp",
    );
    await expect(page.getByTestId("otp-input")).toBeVisible();
    await captureState(page, "login-otp-input");
  });

  // ── Admin: generate invite codes ─────────────────────────────────────────────

  test("capture admin — invite codes section (initial state)", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(ADMIN_PHONE);
    await app.goToDashboard();
    await expect(page.getByTestId("generate-code-button")).toBeEnabled();
    await captureState(page, "admin-invite-codes-initial");
  });

  test("setup — generate invite codes for audit participants", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(ADMIN_PHONE);
    AUDIT_CODES.A = await app.generateInviteCode();
    AUDIT_CODES.B = await app.generateInviteCode();
    AUDIT_CODES.C = await app.generateInviteCode();
    AUDIT_CODES.NEW = await app.generateInviteCode();
    expect(AUDIT_CODES.A.length).toBeGreaterThan(0);
  });

  test("capture admin — invite codes with data", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(ADMIN_PHONE);
    await app.goToDashboard();
    await expect(page.getByTestId("invite-code-list")).toBeVisible();
    await captureState(page, "admin-invite-codes-populated");
  });

  // ── Self-registration flow (new participant) ──────────────────────────────────

  test("capture login — invite code input step", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.reachInviteCodeStep(AUDIT_PHONES.NEW);
    await captureState(page, "login-invite-code-input");
  });

  test("capture login — name input step", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.reachInviteCodeStep(AUDIT_PHONES.NEW);
    await page.getByTestId("invite-code-input").fill(AUDIT_CODES.NEW);
    await app.clickAndWaitForResponse(
      page.getByTestId("submit-invite-code-button"),
      "validate_invite_code",
    );
    await expect(page.getByTestId("legal-name-input")).toBeVisible();
    await captureState(page, "login-name-input");
  });

  // ── Onboarding ────────────────────────────────────────────────────────────────

  test("capture onboarding — branch selection", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.selfRegister(AUDIT_PHONES.A, AUDIT_CODES.A, AUDIT_NAMES.A);
    await expect(page).toHaveURL(/\/onboarding/);
    await expect(page.getByTestId("save-onboarding-button")).toBeEnabled();
    await captureState(page, "onboarding-branch-selection");
    // Complete onboarding so DB state is ready for next tests.
    await app.completeOnboarding("Київ", "1");
  });

  // ── Setup: register and onboard B and C ─────────────────────────────────────

  test("setup — register and onboard participant B", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.selfRegister(AUDIT_PHONES.B, AUDIT_CODES.B, AUDIT_NAMES.B);
    await app.completeOnboarding("Київ", "1");
  });

  test("setup — register and onboard participant C", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.selfRegister(AUDIT_PHONES.C, AUDIT_CODES.C, AUDIT_NAMES.C);
    await app.completeOnboarding("Київ", "1");
  });

  // ── Admin: participants list ──────────────────────────────────────────────────

  test("capture admin — participants list", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(ADMIN_PHONE);
    await app.goToDashboard();
    await expect(page.getByTestId("participant-list")).toBeVisible();
    await captureState(page, "admin-participants-list");
  });

  // ── Admin: season creation ────────────────────────────────────────────────────

  // At this point in the audit flow no season has been created yet, so the admin
  // page shows the participants list and the create-season form together. This is
  // the "no active season" state; create-season-button is visible.
  //
  // Explicit page.goto() is required here: after login, the browser is already
  // on /admin via a redirect. goToDashboard() would skip the navigation and only
  // wait for main — not sufficient to guarantee the SSR Suspense (admin_state
  // Resource) has resolved and injected create-season-button into the HTML.
  // A fresh goto forces a complete SSR round-trip, so the resolved state is
  // present before the enabled check starts.
  test("capture admin — no active season (participants list + create form)", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(ADMIN_PHONE);
    await page.goto("/admin");
    await expect(page.getByTestId("create-season-button")).toBeEnabled();
    await captureState(page, "admin-no-season-create-form-available");
  });

  test("capture admin — unlaunched season (launch button visible)", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(ADMIN_PHONE);
    await app.createSeason(SIGNUP_DEADLINE, CONFIRM_DEADLINE, SEASON_THEME);
    await expect(page.getByTestId("launch-button")).toBeVisible();
    await captureState(page, "admin-season-created-pre-launch");
  });

  // ── Home: no enrollment available (season not yet launched) ──────────────────

  test("capture home — season exists but enrollment not open", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(AUDIT_PHONES.A);
    await app.goHome();
    await captureState(page, "home-enrollment-not-open");
  });

  // ── Admin: launch → signup phase ─────────────────────────────────────────────

  test("capture admin — signup phase (after launch)", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(ADMIN_PHONE);
    await app.launchSeason();
    await app.goToDashboard();
    await expect(page.getByTestId("advance-button")).toBeVisible();
    await captureState(page, "admin-signup-phase");
  });

  // ── Home: enrollment available ────────────────────────────────────────────────

  test("capture home — enrollment available", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(AUDIT_PHONES.A);
    await app.goHome();
    await expect(page.getByTestId("enroll-button")).toBeVisible();
    await captureState(page, "home-enrollment-available");
  });

  // ── Home: enrolled state ──────────────────────────────────────────────────────

  test("capture home — enrolled (after enroll)", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(AUDIT_PHONES.A);
    await app.goHome();
    await app.enrollInSeason("Київ", "1");
    await app.expectEnrolled();
    await captureState(page, "home-enrolled");
  });

  // ── Setup: enroll B and C ─────────────────────────────────────────────────────

  test("setup — enroll participants B and C", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(AUDIT_PHONES.B);
    await app.goHome();
    await app.enrollInSeason("Київ", "1");

    await app.login(AUDIT_PHONES.C);
    await app.goHome();
    await app.enrollInSeason("Київ", "1");
  });

  // ── Phase: advance to confirm (preparation) ──────────────────────────────────

  test("capture admin — confirm phase (preparation)", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(ADMIN_PHONE);
    await app.advanceSeason();
    await app.goToDashboard();
    await expect(page.getByTestId("advance-button")).toBeVisible();
    await captureState(page, "admin-confirm-phase");
  });

  // ── Home: confirmed ready ─────────────────────────────────────────────────────

  test("capture home — confirmed ready (after confirm)", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(AUDIT_PHONES.A);
    await app.goHome();
    await expect(page.getByTestId("confirm-ready-button")).toBeVisible();
    await captureState(page, "home-confirm-ready-available");
    await app.confirmReady();
    await app.expectConfirmed();
    await captureState(page, "home-confirmed-ready");
  });

  // ── Setup: B and C confirm ready ─────────────────────────────────────────────

  test("setup — B and C confirm ready", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(AUDIT_PHONES.B);
    await app.goHome();
    await app.confirmReady();

    await app.login(AUDIT_PHONES.C);
    await app.goHome();
    await app.confirmReady();
  });

  // ── Phase: advance to assignment ─────────────────────────────────────────────

  test("capture admin — assignment phase pre-generate", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(ADMIN_PHONE);
    await app.advanceSeason();
    // Explicit page.goto() required: advanceSeason() leaves the browser on
    // /admin after the POST. goToDashboard() would skip navigation and only
    // wait for main — not sufficient after a phase advance, because the
    // admin_state Resource refetch is async. A fresh SSR goto guarantees
    // the resolved Assignment-phase state (generate-button) is in the HTML.
    await page.goto("/admin");
    await expect(page.getByTestId("generate-button")).toBeVisible();
    await captureState(page, "admin-assignment-phase-pre-generate");
  });

  // ── Admin: assignments generated ─────────────────────────────────────────────

  test("capture admin — assignment generated (cycle visualization)", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(ADMIN_PHONE);
    await app.generateAssignments();
    await expect(page.getByTestId("cycle-visualization")).toBeVisible();
    await captureState(page, "admin-assignment-cycle");
  });

  // ── Admin: swap UI ────────────────────────────────────────────────────────────

  test("capture admin — swap UI visible", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(ADMIN_PHONE);
    await app.goToDashboard();
    await expect(page.getByTestId("override-available")).toBeVisible();
    await captureState(page, "admin-swap-ui");
  });

  // ── Phase: advance to delivery ────────────────────────────────────────────────

  test("capture admin — delivery phase (SMS controls)", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(ADMIN_PHONE);
    await app.advanceSeason();
    // Explicit page.goto() required: same race as assignment-phase test.
    // Delivery phase introduces send-assignment-button; fresh SSR round-trip
    // ensures it is present before the assertion starts.
    await page.goto("/admin");
    await expect(page.getByTestId("send-assignment-button")).toBeVisible();
    await captureState(page, "admin-delivery-phase-sms-controls");
  });

  // ── Admin: SMS report after sending ──────────────────────────────────────────

  test("capture admin — SMS report (after assignment SMS)", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(ADMIN_PHONE);
    await app.triggerSms("assignment");
    await expect(page.getByTestId("sms-report")).toBeVisible();
    await captureState(page, "admin-sms-report");
  });

  // ── Home: participant sees assignment (includes receipt form in same DOM) ─────

  test("capture home — assignment visible to participant", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(AUDIT_PHONES.A);
    await app.goHome();
    await expect(page.getByTestId("recipient-name")).toBeVisible();
    await expect(page.getByTestId("received-button")).toBeVisible();
    await captureState(page, "home-assignment-and-receipt-form");
  });

  // ── Home: receipt confirmed (thank you) ──────────────────────────────────────

  test("capture home — receipt confirmed (thank you)", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(AUDIT_PHONES.A);
    await app.goHome();
    await app.confirmReceipt(true);
    await expect(page.getByTestId("receipt-thanks")).toBeVisible();
    await captureState(page, "home-receipt-confirmed-thanks");
  });

  // ── Setup: B confirms receipt ─────────────────────────────────────────────────

  test("setup — participant B confirms receipt", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(AUDIT_PHONES.B);
    await app.goHome();
    await app.confirmReceipt(true);
  });

  // ── Setup: C confirms receipt ─────────────────────────────────────────────────

  test("setup — participant C confirms receipt", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(AUDIT_PHONES.C);
    await app.goHome();
    await app.confirmReceipt(true);
  });

  // ── Phase: advance to complete ────────────────────────────────────────────────

  test("capture admin — complete phase", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(ADMIN_PHONE);
    await app.advanceSeason();
    // Explicit page.goto() required: same race as other post-advance tests.
    await page.goto("/admin");
    await expect(page.locator("main")).toBeVisible();
    await captureState(page, "admin-season-complete");
  });

  // ── Home: season complete ─────────────────────────────────────────────────────

  test("capture home — season complete", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(AUDIT_PHONES.A);
    await app.goHome();
    await app.expectHomeContent(/complete|завершено/i);
    await captureState(page, "home-season-complete");
  });

  // ── Cancelled season state ────────────────────────────────────────────────────

  test("setup — create and launch a second season for cancel demo", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(ADMIN_PHONE);
    await app.createSeason(futureDatetime(7), futureDatetime(21));
    await app.launchSeason();
  });

  test("capture admin — cancelled season", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(ADMIN_PHONE);
    await app.cancelSeason();
    await app.goToDashboard();
    await expect(page.locator("main")).toBeVisible();
    await captureState(page, "admin-season-cancelled");
  });

  test("capture home — cancelled season state", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(AUDIT_PHONES.A);
    await app.goHome();
    await expect(page.locator("main")).toBeVisible();
    await captureState(page, "home-cancelled-season");
  });

});
