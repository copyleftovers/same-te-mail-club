import { type Page, type Locator, expect } from "@playwright/test";

/**
 * Page Object Model for The Mail Club.
 *
 * Centralizes all selectors and user-action methods.
 * The selector contract is defined in spec/E2E Test Blueprint.md.
 *
 * Conventions (see end2end/README.md for full guide):
 *   - Every ActionForm click uses clickAndWaitForResponse() to ensure the
 *     server function completes before the method returns.
 *   - Methods that represent complete actions (login, createSeason, etc.)
 *     are self-contained — they wait for their own completion signals.
 *   - Methods that represent user choices where the test verifies the outcome
 *     (enrollInSeason, confirmReady, etc.) wait for the POST but leave UI
 *     assertion to the caller.
 *
 * Requires:
 *   SAMETE_TEST_MODE=true  (fixed OTP "000000")
 *   SAMETE_SMS_DRY_RUN=true (no real SMS)
 */

const TEST_OTP = "000000";

export class MailClubPage {
  constructor(public readonly page: Page) {}

  // ── Core helper ──

  /**
   * Click a locator and wait for the server function POST response.
   *
   * Every ActionForm submission triggers a POST. This helper ensures the
   * server has processed the request before the method returns. The listener
   * is set up BEFORE the click so it cannot miss the response.
   *
   * Use this for every ActionForm submit button. Do NOT use page.waitForTimeout().
   */
  async clickAndWaitForResponse(locator: Locator) {
    const responsePromise = this.page.waitForResponse(
      (resp) => resp.request().method() === "POST",
    );
    await locator.click();
    await responsePromise;
  }

  // ── Auth ──

  async login(phone: string) {
    await this.page.goto("/login");

    // Wait for hydration — submit button starts disabled, becomes enabled after WASM loads.
    const sendBtn = this.page.getByRole("button", {
      name: /send|submit|code/i,
    });
    await expect(sendBtn).toBeEnabled();

    await this.page.getByLabel(/phone/i).fill(phone);
    await this.clickAndWaitForResponse(sendBtn);

    // Wait for OTP step to appear before filling.
    const codeInput = this.page.getByLabel(/code/i);
    await expect(codeInput).toBeVisible();
    await codeInput.fill(TEST_OTP);

    const verifyBtn = this.page.getByRole("button", {
      name: /verify|submit|sign/i,
    });
    await this.clickAndWaitForResponse(verifyBtn);

    // Wait for navigation away from login.
    await expect(this.page).not.toHaveURL(/\/login/);
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
    // Wait for hydration before filling inputs.
    await expect(
      this.page.getByRole("button", { name: /save|submit|continue/i }),
    ).toBeEnabled();
    await this.page.getByLabel(/nova poshta|branch|відділення/i).fill(branch);
    await this.page
      .getByRole("button", { name: /save|submit|continue/i })
      .click();
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
    await this.clickAndWaitForResponse(
      this.page.getByTestId("enroll-button"),
    );
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
    await this.clickAndWaitForResponse(
      this.page.getByTestId("confirm-ready-button"),
    );
  }

  async expectConfirmed() {
    await expect(
      this.page.getByTestId("confirm-ready-button"),
    ).not.toBeVisible();
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
      await this.clickAndWaitForResponse(
        this.page.getByTestId("received-button"),
      );
    } else {
      await this.clickAndWaitForResponse(
        this.page.getByTestId("not-received-button"),
      );
    }
  }

  // ── Admin: participants (Story 1.1) ──

  async registerParticipant(phone: string, name: string) {
    await this.page.goto("/admin/participants");
    // Wait for hydration.
    await expect(this.page.getByTestId("register-button")).toBeEnabled();
    await this.page.getByLabel(/phone/i).fill(phone);
    await this.page.getByLabel(/name/i).fill(name);
    await this.clickAndWaitForResponse(
      this.page.getByTestId("register-button"),
    );
  }

  async expectParticipantInList(name: string) {
    await expect(this.page.getByText(name)).toBeVisible();
  }

  async deactivateParticipant(name: string) {
    await this.page.goto("/admin/participants");
    const row = this.page.getByRole("row").filter({ hasText: name });
    await this.clickAndWaitForResponse(row.getByTestId("deactivate-button"));
  }

  // ── Admin: season management (Stories 4.1, 4.2) ──

  async createSeason(
    signupDeadline: string,
    confirmDeadline: string,
    theme?: string,
  ) {
    await this.page.goto("/admin/season");
    // Wait for hydration.
    await expect(this.page.getByTestId("create-season-button")).toBeEnabled();
    await this.page.getByLabel(/signup.*deadline/i).fill(signupDeadline);
    await this.page.getByLabel(/confirm.*deadline/i).fill(confirmDeadline);
    if (theme) {
      await this.page.getByLabel(/theme/i).fill(theme);
    }
    await this.clickAndWaitForResponse(
      this.page.getByTestId("create-season-button"),
    );
    // Wait for the page to transition from create form to active season panel.
    await expect(this.page.getByTestId("launch-button")).toBeVisible();
  }

  async launchSeason() {
    await this.page.goto("/admin/season");
    await this.clickAndWaitForResponse(
      this.page.getByTestId("launch-button"),
    );
    // Wait for launch to complete — advance button appears after launch.
    await expect(this.page.getByTestId("advance-button")).toBeVisible();
  }

  async advanceSeason() {
    await this.page.goto("/admin/season");
    await this.clickAndWaitForResponse(
      this.page.getByTestId("advance-button"),
    );
  }

  async cancelSeason() {
    await this.page.goto("/admin/season");
    await this.clickAndWaitForResponse(
      this.page.getByTestId("cancel-button"),
    );
    // Wait for cancel to complete.
    await expect(this.page.getByTestId("cancel-button")).not.toBeVisible();
  }

  // ── Admin: assignments (Stories 3.1, 3.3) ──

  async generateAssignments() {
    await this.page.goto("/admin/assignments");
    await this.clickAndWaitForResponse(
      this.page.getByTestId("generate-button"),
    );
  }

  async releaseAssignments() {
    await this.clickAndWaitForResponse(
      this.page.getByTestId("release-button"),
    );
  }

  async expectCycleVisualization() {
    await expect(this.page.getByTestId("cycle-visualization")).toBeVisible();
  }

  // ── Admin: SMS triggers (Stories 5.1–5.4) ──

  async triggerSms(
    type: "season-open" | "assignment" | "confirm-nudge" | "receipt-nudge",
  ) {
    await this.page.goto("/admin/sms");
    await this.clickAndWaitForResponse(
      this.page.getByTestId(`send-${type}-button`),
    );
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
