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
    // After OTP verify, the server issues a native 302 redirect chain.
    // For a non-onboarded participant: POST → 302 "/" → (AuthGuard SSR) → 302 "/onboarding".
    // clickAndWaitForResponse (in attemptLogin) resolves on the POST 302 response —
    // the browser is still mid-navigation. toHaveURL auto-retries until the URL
    // no longer matches /login, which covers both the "/" and "/onboarding" destinations.
    await expect(this.page).not.toHaveURL(/\/login/);
  }

  async logout() {
    const logoutBtn = this.page.getByTestId("logout-button");
    await expect(logoutBtn).toBeEnabled();
    await this.clickAndWaitForResponse(logoutBtn, "logout");
    // The logout server function redirects to /, which then redirects to /login
    // (no session). toHaveURL auto-retries until the URL matches /login.
    await expect(this.page).toHaveURL(/\/login/);
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
    await expect(this.page.getByTestId("np-number-input")).toBeEditable();

    const cityInput = this.page.getByTestId("np-city-input");
    const numberInput = this.page.getByTestId("np-number-input");

    await cityInput.fill(city);
    await numberInput.fill(branchNumber);

    // Verify the form fields have the correct values before submitting.
    await expect(cityInput).toHaveValue(city);
    await expect(numberInput).toHaveValue(branchNumber);

    await this.page.getByTestId("save-onboarding-button").click();
    // Wait for redirect to complete - server function redirects to "/".
    // waitForURL resolves as soon as the URL matches — no waitUntil needed.
    // The subsequent toBeVisible on main handles interactivity.
    await this.page.waitForURL("/", { timeout: 15000 });
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
    // Wait for hydration before filling inputs. The enroll-button is disabled
    // (via !hydrated.get()) until WASM hydrates. Filling inputs before hydration
    // risks Leptos patching the DOM and resetting the values before submit.
    await expect(this.page.getByTestId("enroll-button")).toBeEnabled();
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

  // ── Invite codes (Stories 1.1, 1.5, 1.6) ──

  /**
   * Generate a new invite code from the admin page.
   *
   * Navigates to /admin, selects the distributor from the dropdown (by visible
   * name), clicks generate, waits for the generated code display to appear, and
   * returns the raw code string.
   *
   * If distributorName is omitted, the first option in the list is selected.
   * In a freshly seeded DB the only option is the admin ("Організатор").
   */
  async generateInviteCode(distributorName?: string): Promise<string> {
    await this.page.goto("/admin");
    // Wait for hydration — generate-code-button is the gate.
    await expect(this.page.getByTestId("generate-code-button")).toBeEnabled();

    const select = this.page.getByTestId("distributor-select");
    if (distributorName) {
      await select.selectOption({ label: distributorName });
    } else {
      // Select the first non-placeholder option (index 1 — index 0 is the empty prompt).
      const options = await select.locator("option").all();
      if (options.length > 1) {
        const val = await options[1].getAttribute("value");
        if (val) {
          await select.selectOption(val);
        }
      }
    }

    await this.page.getByTestId("generate-code-button").click();
    // Wait for the generated code display to appear.
    await expect(this.page.getByTestId("generated-code-display")).toBeVisible();
    // Read and return the code string.
    const code = await this.page.getByTestId("generated-code-value").textContent();
    return (code ?? "").trim();
  }

  /**
   * Navigate to the invite code step for a new (unregistered) phone.
   *
   * Performs the phone → OTP steps and waits for the invite code input to be
   * ready. Returns without submitting the invite code, so the caller can test
   * various code values (valid, invalid, revoked, used).
   *
   * Precondition: `phone` must NOT have an existing account in the DB.
   */
  async reachInviteCodeStep(phone: string) {
    await this.page.goto("/login");

    const sendBtn = this.page.getByTestId("send-otp-button");
    await expect(sendBtn).toBeEnabled();
    await this.page.getByTestId("phone-input").fill(phone);
    await this.clickAndWaitForResponse(sendBtn, "request_otp");

    await expect(this.page.getByTestId("otp-input")).toBeVisible();
    await this.page.getByTestId("otp-input").fill(TEST_OTP);
    const verifyBtn = this.page.getByTestId("verify-otp-button");
    await this.clickAndWaitForResponse(verifyBtn, "verify_otp_code");
    // After the native POST, the browser follows the 302 redirect back to /login.
    // We cannot use waitForLoadState("domcontentloaded") here because the redirect
    // goes back to the same URL (/login). waitForLoadState would resolve immediately
    // since the page already reached domcontentloaded on the initial load.
    // Instead, wait for the invite-code-step div to become visible — this only
    // happens when the reloaded /login SSR sees the pending_phone cookie and
    // renders is_pending=true (step 3).
    await expect(this.page.getByTestId("invite-code-step")).toBeVisible();

    // Wait for hydration — submit button is the gate for the invite code step.
    await expect(this.page.getByTestId("submit-invite-code-button")).toBeEnabled();
    // Extra safety: ensure the input itself is attached and editable.
    // Hydration may replace the SSR input node after the button becomes enabled.
    await expect(this.page.getByTestId("invite-code-input")).toBeEditable();
  }

  /**
   * Self-register a new participant using an invite code.
   *
   * Step 1: Enter phone on login page, request OTP.
   * Step 2: Enter OTP (test mode: always "000000"). Native form POST → server
   *         sets pending_phone cookie and redirects back to /login.
   * Step 3: Enter invite code. ActionForm calls validate_invite_code.
   * Step 4: Enter name. ActionForm calls register_with_code → redirects to /onboarding.
   *
   * Completes at /onboarding (caller can then call completeOnboarding).
   */
  async selfRegister(phone: string, code: string, name: string) {
    // Steps 1 + 2: Navigate to invite code step.
    await this.reachInviteCodeStep(phone);

    // Step 3: Enter invite code and submit.
    // fill() is safe because reachInviteCodeStep confirmed toBeEditable().
    await this.page.getByTestId("invite-code-input").fill(code);
    // Use clickAndWaitForResponse so we know the server processed the code
    // before asserting on the DOM change that follows. A plain click() leaves a
    // race window where fill() may target a pre-hydration element that WASM
    // replaces during the server round-trip, causing the server to receive an
    // empty code string.
    await this.clickAndWaitForResponse(
      this.page.getByTestId("submit-invite-code-button"),
      "validate_invite_code",
    );
    // Wait for name collection step to appear (code validated successfully).
    await expect(this.page.getByTestId("legal-name-input")).toBeVisible();
    // Wait for the name step's submit button to be enabled (hydration gate for
    // the register_with_code ActionForm).
    await expect(this.page.getByTestId("create-account-button")).toBeEnabled();

    // Step 4: Name collection — fill name and submit via native POST.
    // register_with_code uses native <form method="post"> (not ActionForm) because
    // it must set an HttpOnly session cookie, which requires a full HTTP response.
    await this.page.getByTestId("legal-name-input").fill(name);
    await this.clickAndWaitForResponse(
      this.page.getByTestId("create-account-button"),
      "register_with_code",
    );
    // After the native POST → 302 → /onboarding, toHaveURL auto-retries
    // until the URL matches. No waitUntil needed — URL change implies the
    // response has arrived.
    await expect(this.page).toHaveURL(/\/onboarding/);
  }

  /**
   * Revoke an unused invite code from the admin page.
   *
   * Navigates to /admin, finds the invite-code-row containing the code string,
   * clicks its revoke button, and waits for the status badge to change.
   */
  async revokeInviteCode(codeString: string) {
    await this.page.goto("/admin");
    // Wait for the invite code list to be populated.
    await expect(this.page.getByTestId("invite-code-list")).toBeVisible();
    const row = this.page.getByTestId("invite-code-row").filter({
      has: this.page.getByTestId("invite-code-cell").filter({ hasText: codeString }),
    });
    await expect(row).toBeVisible();
    await row.getByTestId("invite-code-revoke-button").click();
    // Wait for the status badge in this row to reflect the revoked state.
    // The revoke action updates the list resource, which re-renders the row.
    await expect(row.getByTestId("invite-code-status-badge")).not.toHaveAttribute(
      "data-status",
      "unused",
    );
  }

  /**
   * Assert the status badge of a specific invite code row.
   *
   * Finds the invite-code-row containing the code string and asserts that the
   * invite-code-status-badge has the expected data-status attribute value.
   */
  async expectInviteCodeStatus(codeString: string, expectedStatus: string) {
    const row = this.page.getByTestId("invite-code-row").filter({
      has: this.page.getByTestId("invite-code-cell").filter({ hasText: codeString }),
    });
    await expect(row.getByTestId("invite-code-status-badge")).toHaveAttribute(
      "data-status",
      expectedStatus,
    );
  }

  /**
   * Assert that the invite code input step is visible.
   *
   * Used to verify that the user has landed on the invite code step of the
   * self-registration flow.
   */
  async expectSelfRegistrationPrompt() {
    await expect(this.page.getByTestId("invite-code-input")).toBeVisible();
  }

  async expectParticipantInList(name: string) {
    await expect(this.page.getByTestId("participant-name-cell").filter({hasText: name})).toBeVisible();
  }

  async deactivateParticipant(name: string) {
    await this.page.goto("/admin");
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
    await this.page.goto("/admin");
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
    await this.page.goto("/admin");
    await this.clickAndWaitForResponse(
      this.page.getByTestId("launch-button"),
      "launch_season",
    );
    // Wait for launch to complete — advance button appears after launch.
    await expect(this.page.getByTestId("advance-button")).toBeVisible();
  }

  async advanceSeason() {
    await this.page.goto("/admin");
    // The advance button stays visible for 3 of 4 transitions (Enrollment →
    // Preparation → Assignment → Delivery), so there is no single reliable DOM
    // signal for all callers. The URL-filtered POST wait is sufficient: callers
    // always navigate away via goToDashboard() and assert on the next page state.
    await this.clickAndWaitForResponse(
      this.page.getByTestId("advance-button"),
      "advance",
    );
  }

  async cancelSeason() {
    await this.page.goto("/admin");
    await this.page.getByTestId("cancel-button").click();
    await expect(this.page.getByTestId("cancel-confirmation")).toBeVisible();
    await this.clickAndWaitForResponse(
      this.page.getByTestId("cancel-confirm-button"),
      "cancel_season",
    );
    await expect(this.page.getByTestId("cancel-confirm-button")).not.toBeVisible();
  }

  // ── Admin: assignments (Stories 3.1, 3.3) ──

  async generateAssignments() {
    await this.page.goto("/admin");
    await this.page.getByTestId("generate-button").click();
    // Wait for refetch to complete — cycle visualization appears.
    await expect(this.page.getByTestId("cycle-visualization")).toBeVisible();
  }

  async swapAssignment(senderNameA: string, senderNameB: string) {
    await this.page.goto("/admin");
    // Wait for hydration — swap button disabled until WASM loads.
    await expect(this.page.getByTestId("swap-button")).toBeEnabled();
    // Select by label (participant name) — the <select> options use sender names as text.
    await this.page.getByTestId("sender-a-input").selectOption({ label: senderNameA });
    await this.page.getByTestId("sender-b-input").selectOption({ label: senderNameB });
    // The swap action returns (). The preview Resource refetches via swap_action.version()
    // and re-renders the cycle visualization — use URL-filtered POST wait then
    // assert cycle-visualization visible to confirm refetch completed.
    await this.clickAndWaitForResponse(
      this.page.getByTestId("swap-button"),
      "swap_assignment",
    );
    await expect(this.page.getByTestId("cycle-visualization")).toBeVisible();
  }

  async expectCycleVisualization() {
    await expect(this.page.getByTestId("cycle-visualization")).toBeVisible();
  }

  // ── Admin: SMS triggers (Stories 5.1–5.4) ──

  async triggerSms(
    type: "season-open" | "assignment" | "confirm-nudge" | "receipt-nudge",
  ) {
    await this.page.goto("/admin");
    await this.page.getByTestId(`send-${type}-button`).click();
    // Wait for refetch to complete — SMS report appears.
    await expect(this.page.getByTestId("sms-report")).toBeVisible();
  }

  async expectSmsReport() {
    await expect(this.page.getByTestId("sms-report")).toBeVisible();
  }

  // ── Admin: SMS counts (Story 4.4) ──

  async expectSmsCountVisible(testid: string): Promise<void> {
    await expect(this.page.getByTestId(testid)).toBeVisible();
  }

  async expectSmsCount(testid: string, expectedText: string): Promise<void> {
    await expect(this.page.getByTestId(testid)).toContainText(expectedText);
  }

  // ── Admin: season advance gating (Story 4.7) ──

  async expectAdvanceBlocked(): Promise<void> {
    await expect(this.page.getByTestId("advance-blocked-hint")).toBeVisible();
    await expect(this.page.getByTestId("advance-button")).toBeDisabled();
  }

  async expectAdvanceEnabled(): Promise<void> {
    await expect(
      this.page.getByTestId("advance-blocked-hint"),
    ).not.toBeVisible();
    await expect(this.page.getByTestId("advance-button")).toBeEnabled();
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
