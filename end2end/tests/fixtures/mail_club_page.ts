import { type Page, type Locator, expect } from "@playwright/test";

/**
 * Page Object Model for The Mail Club.
 *
 * Centralizes all selectors and user-action methods.
 * The selector contract is defined in spec/E2E Test Blueprint.md.
 *
 * Conventions (see end2end/README.md for full guide):
 *   - Primary pattern: click() + element visibility wait. Playwright auto-waits
 *     for the button to be enabled (hydration gate), and the visibility assertion
 *     auto-retries until the Resource refetch completes and the DOM updates.
 *   - clickAndWaitForResponse() is reserved for cases with no visible DOM change
 *     after the action (advanceSeason) and for the login flow (native form POST).
 *   - Methods that represent complete actions (login, createSeason, etc.)
 *     are self-contained — they wait for their own completion signals.
 *   - Methods that represent user choices where the test verifies the outcome
 *     (enrollInSeason, confirmReady, etc.) self-contain their completion wait
 *     via element visibility assertions.
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
   * Click a locator and wait for a POST response.
   *
   * Use this only when there is no visible DOM change to wait for after the
   * action (e.g., advanceSeason navigates away immediately, login uses a native
   * form POST). For all other cases, prefer click() + element visibility wait.
   *
   * The listener is set up BEFORE the click so it cannot miss the response.
   * The optional urlHint filters to a specific endpoint, preventing the listener
   * from accidentally matching a concurrent Resource refetch POST.
   */
  async clickAndWaitForResponse(locator: Locator, urlHint?: string) {
    const responsePromise = this.page.waitForResponse(
      (resp) =>
        resp.request().method() === "POST" &&
        (urlHint ? resp.url().includes(urlHint) : true),
    );
    await locator.click();
    await responsePromise;
  }

  // ── Auth ──

  // Performs the full two-step login flow (request OTP → verify OTP) without
  // asserting the outcome. The OTP step always appears — the server never reveals
  // whether a phone is registered (privacy invariant). Rejection happens at the
  // verify step, after which the caller asserts the expected URL.
  async attemptLogin(phone: string) {
    await this.page.goto("/login");
    const sendBtn = this.page.getByTestId("send-otp-button");
    await expect(sendBtn).toBeEnabled();
    await this.page.getByTestId("phone-input").fill(phone);
    await this.clickAndWaitForResponse(sendBtn, "request_otp");
    const codeInput = this.page.getByTestId("otp-input");
    await expect(codeInput).toBeVisible();
    await codeInput.fill(TEST_OTP);
    const verifyBtn = this.page.getByTestId("verify-otp-button");
    await this.clickAndWaitForResponse(verifyBtn, "verify_otp_code");
  }

  async login(phone: string) {
    await this.attemptLogin(phone);
    await expect(this.page).not.toHaveURL(/\/login/);
    // The OTP verify form is a native POST that triggers a 302 redirect.
    // waitForResponse resolves on the 302, but the browser is still navigating
    // to the redirect target (e.g. /admin). Without this wait, a subsequent
    // page.goto() races with the in-progress redirect navigation, which can
    // cause the server's SSR response to never complete (the goto cancels the
    // redirect mid-stream, leaving Suspense boundaries unresolved).
    // Use "domcontentloaded" not "load" — we only need the HTML committed,
    // not the 14MB dev WASM bundle fully downloaded.
    await this.page.waitForLoadState("domcontentloaded");
  }

  async logout() {
    const logoutBtn = this.page.getByTestId("logout-button");
    await expect(logoutBtn).toBeEnabled();
    await this.clickAndWaitForResponse(logoutBtn, "logout");
    // The logout server function redirects to /, which then redirects to /login
    // (no session). Wait for the final destination directly — waiting for the
    // intermediate "/" would miss the second redirect.
    await expect(this.page).toHaveURL(/\/login/);
    await this.page.waitForLoadState("domcontentloaded");
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

  async completeOnboarding(city: string, branchNumber: string) {
    // Wait for hydration before filling inputs.
    await expect(
      this.page.getByTestId("save-onboarding-button"),
    ).toBeEnabled();

    const cityInput = this.page.getByTestId("np-city-input");
    const numberInput = this.page.getByTestId("np-number-input");

    await cityInput.fill(city);
    await numberInput.fill(branchNumber);

    // Verify the form fields have the correct values before submitting.
    await expect(cityInput).toHaveValue(city);
    await expect(numberInput).toHaveValue(branchNumber);

    await this.page.getByTestId("save-onboarding-button").click();
    // Wait for redirect to complete - server function redirects to "/".
    await this.page.waitForURL("/", { timeout: 15000 });
    // The server's complete_onboarding does a 302 redirect to "/". waitForURL
    // resolves when the URL changes, but the browser may still be loading the
    // redirect target. Without this wait, a subsequent navigation races with
    // the in-progress page load.
    await this.page.waitForLoadState("domcontentloaded");
  }

  // ── Home Screen ──

  async goHome() {
    // Skip navigation if already on "/". After login(), the participant is
    // already redirected to "/" with the page fully loaded. Calling goto("/")
    // again forces a redundant full SSR reload + 14MB WASM re-download in dev
    // mode, which intermittently exceeds the 15s navigation timeout.
    const currentUrl = new URL(this.page.url());
    if (currentUrl.pathname !== "/") {
      await this.page.goto("/");
    }
    // Wait for hydration by checking that the main content container is fully rendered.
    // The page has many states (NoSeason, EnrollmentOpen, Enrolled, etc.), many of which
    // have no buttons at all (50% of states). Waiting for "first button enabled" fails
    // on these states. Instead, wait for <main> to be stable — it exists in all states
    // and proves the page is interactive (hydration complete).
    await expect(this.page.locator("main")).toBeVisible({ timeout: 10_000 });
  }

  async expectHomeContent(text: string | RegExp) {
    await expect(this.page.locator("main")).toContainText(text);
  }

  // ── Season enrollment (Story 2.1) ──

  async enrollInSeason(city?: string, branchNumber?: string) {
    if (city) {
      await this.page.getByTestId("np-city-input").fill(city);
    }
    if (branchNumber) {
      await this.page.getByTestId("np-number-input").fill(branchNumber);
    }
    await this.page.getByTestId("enroll-button").click();
    // Wait for refetch to complete — enroll button disappears when enrolled.
    await expect(this.page.getByTestId("enroll-button")).not.toBeVisible();
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
    // Wait for refetch to complete — confirm button disappears when confirmed.
    await expect(
      this.page.getByTestId("confirm-ready-button"),
    ).not.toBeVisible();
  }

  async expectConfirmed() {
    await expect(
      this.page.getByTestId("confirm-ready-button"),
    ).not.toBeVisible();
  }

  // ── Assignment view (Story 2.3) ──

  async revealAssignment() {
    const envelope = this.page.getByTestId("reveal-envelope");
    await envelope.click();
    // Wait for content to be visible after animation
    await expect(this.page.getByTestId("recipient-name")).toBeVisible();
  }

  async expectAssignmentVisible() {
    // Click envelope to reveal (idempotent - CSS handles already-expanded)
    await this.revealAssignment();
    await expect(this.page.getByTestId("recipient-name")).toBeVisible();
    await expect(this.page.getByTestId("recipient-phone")).toBeVisible();
    await expect(this.page.getByTestId("recipient-branch")).toBeVisible();
  }

  async getAssignment() {
    // Ensure envelope is revealed before reading text content
    await this.revealAssignment();
    return {
      name: await this.page.getByTestId("recipient-name").textContent(),
      phone: await this.page.getByTestId("recipient-phone").textContent(),
      branch: await this.page.getByTestId("recipient-branch").textContent(),
    };
  }

  // ── Receipt confirmation (Story 2.4) ──

  async confirmReceipt(received: boolean, note?: string) {
    if (note) {
      await this.page.getByTestId("receipt-note-input").fill(note);
    }
    if (received) {
      await this.page.getByTestId("received-button").click();
      // Wait for refetch to complete — completion signal appears.
      await expect(this.page.getByTestId("receipt-thanks")).toBeVisible();
    } else {
      await this.page.getByTestId("not-received-button").click();
      // Wait for refetch to complete — completion signal appears.
      await expect(this.page.getByTestId("receipt-thanks")).toBeVisible();
    }
  }

  // ── Admin: participants (Story 1.1) ──

  async registerParticipant(phone: string, name: string) {
    await this.page.goto("/admin/participants");
    // Wait for hydration.
    await expect(this.page.getByTestId("register-button")).toBeEnabled();
    await this.page.getByTestId("reg-phone-input").fill(phone);
    await this.page.getByTestId("reg-name-input").fill(name);
    await this.page.getByTestId("register-button").click();
    // Wait for either success (name appears) or error (error message).
    // Whichever happens first, the action has completed.
    await Promise.race([
      expect(this.page.getByTestId("participant-name-cell").filter({hasText: name})).toBeVisible(),
      expect(this.page.getByTestId("action-error")).toBeVisible(),
    ]);
  }

  async expectParticipantInList(name: string) {
    await expect(this.page.getByTestId("participant-name-cell").filter({hasText: name})).toBeVisible();
  }

  async deactivateParticipant(name: string) {
    await this.page.goto("/admin/participants");
    const row = this.page.getByTestId("participant-row").filter({ hasText: name });
    // Wait for hydration — the row being visible means data loaded and the page rendered.
    await expect(row).toBeVisible();
    await row.getByTestId("deactivate-button").click();
    // Wait for refetch to complete — inactive status appears.
    await expect(this.page.getByTestId("inactive-status")).toBeVisible();
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
    await this.page.getByTestId("signup-deadline-input").fill(signupDeadline);
    await this.page.getByTestId("confirm-deadline-input").fill(confirmDeadline);
    if (theme) {
      await this.page.getByTestId("theme-input").fill(theme);
    }
    await this.clickAndWaitForResponse(
      this.page.getByTestId("create-season-button"),
      "create_season",
    );
    // Wait for the page to transition from create form to active season panel.
    await expect(this.page.getByTestId("launch-button")).toBeVisible();
  }

  async launchSeason() {
    await this.page.goto("/admin/season");
    await this.clickAndWaitForResponse(
      this.page.getByTestId("launch-button"),
      "launch_season",
    );
    // Wait for launch to complete — advance button appears after launch.
    await expect(this.page.getByTestId("advance-button")).toBeVisible();
  }

  async advanceSeason() {
    await this.page.goto("/admin/season");
    // The advance action triggers a Resource refetch that re-renders the season
    // panel (phase label changes, buttons may appear/disappear). However, the
    // phase label has no data-testid, and the advance button stays visible for
    // 3 of 4 transitions — so there is no single reliable DOM signal. Use the
    // URL-filtered POST wait, then waitForLoadState to let the refetch settle
    // before the caller navigates away.
    await this.clickAndWaitForResponse(
      this.page.getByTestId("advance-button"),
      "advance",
    );
    await this.page.waitForLoadState("domcontentloaded");
  }

  async cancelSeason() {
    await this.page.goto("/admin/season");
    await this.clickAndWaitForResponse(
      this.page.getByTestId("cancel-button"),
      "cancel_season",
    );
    // Wait for cancel to complete.
    await expect(this.page.getByTestId("cancel-button")).not.toBeVisible();
  }

  // ── Admin: assignments (Stories 3.1, 3.3) ──

  async generateAssignments() {
    await this.page.goto("/admin/assignments");
    await this.page.getByTestId("generate-button").click();
    // Wait for refetch to complete — cycle visualization appears.
    await expect(this.page.getByTestId("cycle-visualization")).toBeVisible();
  }

  async releaseAssignments() {
    await this.page.getByTestId("release-button").click();
    // Wait for refetch to complete — released status text appears.
    await expect(
      this.page.getByTestId("released-status"),
    ).toBeVisible();
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
    // Wait for refetch to complete — SMS report appears.
    await expect(this.page.getByTestId("sms-report")).toBeVisible();
  }

  async expectSmsReport() {
    await expect(this.page.getByTestId("sms-report")).toBeVisible();
  }

  // ── Admin: dashboard ──

  async goToDashboard() {
    // Skip navigation if already on "/admin". After login(), the admin is
    // already redirected to "/admin" with the page fully loaded. Calling
    // goto("/admin") again forces a redundant full SSR reload + WASM
    // re-download in dev mode, which can exceed the navigation timeout.
    const currentUrl = new URL(this.page.url());
    if (currentUrl.pathname !== "/admin") {
      await this.page.goto("/admin");
    }
    // Wait for the main content container to be fully rendered, confirming
    // navigation and hydration are complete.
    await expect(this.page.locator("main")).toBeVisible({ timeout: 10_000 });
  }

  async expectDashboardContent(text: string | RegExp) {
    await expect(this.page.locator("main")).toContainText(text);
  }
}
