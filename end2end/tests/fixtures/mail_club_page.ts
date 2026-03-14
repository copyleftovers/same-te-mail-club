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
