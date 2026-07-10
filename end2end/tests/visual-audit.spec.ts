import { execSync } from "child_process";
import { type Page } from "@playwright/test";
import { test, expect } from "./fixtures/cached-context";
import { MailClubPage } from "./fixtures/mail_club_page";
import * as fs from "fs";
import * as path from "path";

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
 *   screenshots/light-desktop/{area}-{state-slug}[__{variant}].png
 *   screenshots/light-mobile/{area}-{state-slug}[__{variant}].png
 *   screenshots/dark-desktop/{area}-{state-slug}[__{variant}].png
 *   screenshots/dark-mobile/{area}-{state-slug}[__{variant}].png
 *   screenshots/sections/{area}-{state-slug}__{testid}.png
 *   screenshots/INDEX.md (generated in afterAll)
 *
 * Naming: {area}-{state-slug}[__{variant}].png  — stable across runs,
 * no sequential counter. A failed capture produces a missing file, not a
 * shifted index for all subsequent captures.
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

// Long-content name and address for participant A — exercises text-overflow
// in cards, badges, .deadline, headings, and table cells.
const LONG_NAME_A = "Олександра-Вікторія Кравченко-Мельниченко";
const LONG_CITY_A = "Кривий Ріг Дніпропетровська область";
const LONG_BRANCH_A = "128";

// ── Deadline helpers ───────────────────────────────────────────────────────────

function futureDatetime(daysFromNow: number): string {
  const d = new Date(Date.now() + daysFromNow * 24 * 60 * 60 * 1000);
  return d.toISOString().slice(0, 16);
}

const SIGNUP_DEADLINE = futureDatetime(7);
const CONFIRM_DEADLINE = futureDatetime(21);
// Long theme string — exercises theme display in season summary and admin headings.
const SEASON_THEME =
  "Аудит — Перший сезон: Книги про мандри та подорожі навколо світу";

// ── Invite codes captured during the run ──────────────────────────────────────

const AUDIT_CODES = { A: "", B: "", C: "", NEW: "" };

// ── Manifest accumulator ───────────────────────────────────────────────────────
// Populated during capture calls; written to INDEX.md in afterAll.

interface ManifestEntry {
  file: string;
  stateId: string;
  route: string;
}

const MANIFEST: ManifestEntry[] = [];

function addManifestEntry(file: string, stateId: string, route: string): void {
  MANIFEST.push({ file, stateId, route });
}

// ── Screenshot directories ─────────────────────────────────────────────────────

const SCREENSHOT_DIRS = [
  "screenshots/light-desktop",
  "screenshots/light-mobile",
  "screenshots/dark-desktop",
  "screenshots/dark-mobile",
  "screenshots/sections",
] as const;

// ── Viewport + timing constants ────────────────────────────────────────────────

const MOBILE_VIEWPORT = { width: 375, height: 812 } as const;
const DESKTOP_VIEWPORT = { width: 1280, height: 800 } as const;
// Documented exception to the waitForTimeout ban: paint-timing settle for
// screenshot captures only (not an assertion wait). Pre-existing pattern.
const LAYOUT_REFLOW_MS = 150;

// ── Core capture helpers ───────────────────────────────────────────────────────

/**
 * Capture all four full-page variants (light/dark × desktop/mobile) for a
 * named app state. Adds four INDEX.md entries.
 */
async function captureState(
  page: Page,
  name: string,
  meta: { stateId?: string; route?: string } = {},
): Promise<void> {
  const stateId = meta.stateId ?? name;
  const route = meta.route ?? "";

  // ── Light captures ──
  await page.emulateMedia({ colorScheme: "light" });

  await page.setViewportSize(MOBILE_VIEWPORT);
  await page.waitForTimeout(LAYOUT_REFLOW_MS);
  await page.screenshot({
    path: `screenshots/light-mobile/${name}.png`,
    fullPage: true,
  });
  addManifestEntry(`light-mobile/${name}.png`, stateId, route);

  await page.setViewportSize(DESKTOP_VIEWPORT);
  await page.waitForTimeout(LAYOUT_REFLOW_MS);
  await page.screenshot({
    path: `screenshots/light-desktop/${name}.png`,
    fullPage: true,
  });
  addManifestEntry(`light-desktop/${name}.png`, stateId, route);

  // ── Dark captures ──
  await page.emulateMedia({ colorScheme: "dark" });
  await page.waitForTimeout(LAYOUT_REFLOW_MS);

  await page.setViewportSize(DESKTOP_VIEWPORT);
  await page.waitForTimeout(LAYOUT_REFLOW_MS);
  await page.screenshot({
    path: `screenshots/dark-desktop/${name}.png`,
    fullPage: true,
  });
  addManifestEntry(`dark-desktop/${name}.png`, stateId, route);

  await page.setViewportSize(MOBILE_VIEWPORT);
  await page.waitForTimeout(LAYOUT_REFLOW_MS);
  await page.screenshot({
    path: `screenshots/dark-mobile/${name}.png`,
    fullPage: true,
  });
  addManifestEntry(`dark-mobile/${name}.png`, stateId, route);

  // Reset to light so subsequent interactions use the default color scheme.
  await page.emulateMedia({ colorScheme: "light" });
}

/**
 * Capture a single element-state variant (error, focus) in all four
 * light+dark × desktop+mobile combinations.
 *
 * `captureElementState` adds dark variants; the `field-error` dark-mode gap
 * from deferred_items is closed here.
 */
async function captureElementState(
  page: Page,
  name: string,
  suffix: string,
  meta: { stateId?: string; route?: string } = {},
): Promise<void> {
  const stateId = meta.stateId ?? name;
  const route = meta.route ?? "";
  const filename = `${name}__${suffix}.png`;

  for (const scheme of ["light", "dark"] as const) {
    await page.emulateMedia({ colorScheme: scheme });
    await page.waitForTimeout(LAYOUT_REFLOW_MS);
    const dir = scheme === "light" ? "light" : "dark";

    await page.setViewportSize(DESKTOP_VIEWPORT);
    await page.waitForTimeout(LAYOUT_REFLOW_MS);
    await page.screenshot({
      path: `screenshots/${dir}-desktop/${filename}`,
      fullPage: true,
    });
    addManifestEntry(`${dir}-desktop/${filename}`, stateId, route);

    await page.setViewportSize(MOBILE_VIEWPORT);
    await page.waitForTimeout(LAYOUT_REFLOW_MS);
    await page.screenshot({
      path: `screenshots/${dir}-mobile/${filename}`,
      fullPage: true,
    });
    addManifestEntry(`${dir}-mobile/${filename}`, stateId, route);
  }

  // Reset to light.
  await page.emulateMedia({ colorScheme: "light" });
}

/**
 * Capture a single admin section by testid, desktop/light only.
 * Section crops are detail close-ups; full-page shots cover both modes.
 */
async function captureSection(
  page: Page,
  stateName: string,
  sectionTestid: string,
): Promise<void> {
  await page.setViewportSize(DESKTOP_VIEWPORT);
  await page.waitForTimeout(LAYOUT_REFLOW_MS);
  const filename = `${stateName}__${sectionTestid}.png`;
  await page.getByTestId(sectionTestid).screenshot({
    path: `screenshots/sections/${filename}`,
  });
  addManifestEntry(`sections/${filename}`, stateName, "");
}

// ── Directory creation + stale-file cleanup ────────────────────────────────────

test.beforeAll(() => {
  for (const dir of SCREENSHOT_DIRS) {
    fs.mkdirSync(dir, { recursive: true });
  }
  // Remove stale .png files so orphans from prior runs don't accumulate.
  for (const dir of SCREENSHOT_DIRS) {
    for (const file of fs.readdirSync(dir)) {
      if (file.endsWith(".png")) {
        fs.rmSync(path.join(dir, file));
      }
    }
  }
});

// ── INDEX.md generation ────────────────────────────────────────────────────────

test.afterAll(() => {
  const timestamp = new Date().toISOString();
  let head = "(unknown)";
  try {
    head = execSync("git rev-parse HEAD", { cwd: path.resolve("..") })
      .toString()
      .trim();
  } catch {
    // non-fatal: git may not be available in all harness environments
  }

  const rows = MANIFEST.map(
    (e) =>
      `| ${e.file} | ${e.stateId} | ${e.route} |`,
  ).join("\n");

  const content = [
    "# Screenshot Index",
    `Generated: ${timestamp} | HEAD: ${head}`,
    "",
    "| File | State | Route |",
    "|------|-------|-------|",
    rows,
  ].join("\n");

  fs.writeFileSync("screenshots/INDEX.md", content + "\n");
});

// ── Visual Audit ──────────────────────────────────────────────────────────────

test.describe.serial("Visual Audit", () => {

  // ── Login flow (admin) ───────────────────────────────────────────────────────

  test("capture login — phone input", async ({ page }) => {
    await page.context().clearCookies();
    await page.goto("/login");
    await expect(page.getByTestId("send-otp-button")).toBeEnabled();
    // ── Focus state: focus the phone input to capture :focus-visible ring ──
    await page.getByTestId("phone-input").focus();
    await captureElementState(page, "login-phone-step", "focus");
    // Blur so the ring is absent from the subsequent full-page captures.
    await page.getByTestId("phone-input").blur();
    await captureState(page, "login-phone-step", { stateId: "L1", route: "/login" });
    // ── Error state: invalid phone ──
    await page.getByTestId("phone-input").fill("123");
    await page.getByTestId("send-otp-button").click();
    await expect(page.getByTestId("phone-error")).toBeVisible();
    await captureElementState(page, "login-phone-step", "error");
  });

  test("capture login — otp input", async ({ page }) => {
    const app = new MailClubPage(page);
    // Clear any lingering session/pending cookies from the previous test so
    // the login page SSR always starts from the phone-input step (step 1).
    await page.context().clearCookies();
    await page.goto("/login");
    // Step-1 check: phone-input visible confirms the phone step is rendered (not OTP).
    await expect(page.getByTestId("phone-input")).toBeVisible();
    // Hydration gate: send-otp-button is disabled until WASM hydrates; toBeEnabled()
    // is the README-prescribed wait before filling inputs (end2end/README.md §The Hydration Gate).
    await expect(page.getByTestId("send-otp-button")).toBeEnabled();
    await page.getByTestId("phone-input").fill(ADMIN_PHONE);
    await app.clickAndWaitForResponse(
      page.getByTestId("send-otp-button"),
      "request_otp",
    );
    await expect(page.getByTestId("otp-input")).toBeVisible();
    await captureState(page, "login-otp-step", { stateId: "L5", route: "/login" });
    // ── L7: resend affordance active — distinct named capture per design ──
    await expect(page.getByTestId("resend-otp-button")).toBeVisible();
    await captureState(page, "login-otp-step-resend-active", { stateId: "L7", route: "/login" });
    // ── Error state: wrong OTP reveals otp-error ──
    // The native POST form processes the bad code server-side and 302-redirects
    // to /login?otp_error=1. The browser follows the redirect and re-renders the
    // OTP step with is_otp_error=true. Wait for the URL change then assert the
    // error text is present — both are auto-retrying assertions, no waitForTimeout.
    await page.getByTestId("otp-input").fill("999999");
    await page.getByTestId("verify-otp-button").click();
    await expect(page).toHaveURL(/otp_error/);
    await expect(page.getByTestId("otp-error")).not.toBeEmpty();
    await captureElementState(page, "login-otp-step", "error");
  });

  // ── Admin: generate invite codes ─────────────────────────────────────────────

  test("capture admin — invite codes section (initial state)", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(ADMIN_PHONE);
    await app.goToDashboard();
    await expect(page.getByTestId("generate-code-button")).toBeEnabled();
    await captureState(page, "admin-invite-codes-initial", { stateId: "A60", route: "/admin" });
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

  test("capture admin — invite codes with data + filter active", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(ADMIN_PHONE);
    await app.goToDashboard();
    await expect(page.getByTestId("invite-code-list")).toBeVisible();
    await captureState(page, "admin-invite-codes-populated", { stateId: "A61a", route: "/admin" });
    // ── A62: filter active — fill partial code to narrow the list ──
    const filterInput = page.getByTestId("invite-code-filter-input");
    await filterInput.fill(AUDIT_CODES.A.slice(0, 4));
    await page.waitForTimeout(LAYOUT_REFLOW_MS);
    await captureState(page, "admin-invite-codes-filter-active", { stateId: "A62", route: "/admin" });
    // Clear filter so downstream tests see unfiltered list.
    await filterInput.fill("");
  });

  // ── Global: mobile menu open ──────────────────────────────────────────────────

  test("capture global — mobile menu open", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(ADMIN_PHONE);
    await app.goToDashboard();
    // Set mobile viewport before clicking so the menu button is rendered.
    await page.setViewportSize(MOBILE_VIEWPORT);
    await page.waitForTimeout(LAYOUT_REFLOW_MS);
    await page.getByTestId("menu-toggle").click();
    await page.waitForTimeout(LAYOUT_REFLOW_MS);
    // Capture light only (menu overlay is the same element in both modes;
    // full-page dark shots cover dark rendering in every state).
    await page.emulateMedia({ colorScheme: "light" });
    await page.screenshot({
      path: "screenshots/light-mobile/global-mobile-menu-open.png",
      fullPage: true,
    });
    addManifestEntry("light-mobile/global-mobile-menu-open.png", "G7", "/admin");
    await page.emulateMedia({ colorScheme: "dark" });
    await page.waitForTimeout(LAYOUT_REFLOW_MS);
    await page.screenshot({
      path: "screenshots/dark-mobile/global-mobile-menu-open.png",
      fullPage: true,
    });
    addManifestEntry("dark-mobile/global-mobile-menu-open.png", "G7", "/admin");
    await page.emulateMedia({ colorScheme: "light" });
    await page.setViewportSize(DESKTOP_VIEWPORT);
  });

  // ── Global: 404 fallback ──────────────────────────────────────────────────────

  test("capture global — 404 fallback", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(ADMIN_PHONE);
    await page.goto("/this-route-does-not-exist");
    await expect(page.getByTestId("not-found")).toBeVisible();
    await captureState(page, "global-404-fallback", { stateId: "G13", route: "/this-route-does-not-exist" });
  });

  // ── Self-registration flow (new participant) ──────────────────────────────────

  test("capture login — invite code input step + error", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.reachInviteCodeStep(AUDIT_PHONES.NEW);
    await captureState(page, "login-invite-code-step", { stateId: "L10", route: "/login" });
    // ── Error: submit an empty invite code ──
    await app.clickAndWaitForResponse(
      page.getByTestId("submit-invite-code-button"),
      "validate_invite_code",
    );
    await expect(page.getByTestId("invite-code-error")).not.toBeEmpty();
    await captureElementState(page, "login-invite-code-step", "error");
  });

  test("capture login — name input step + error", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.reachInviteCodeStep(AUDIT_PHONES.NEW);
    await page.getByTestId("invite-code-input").fill(AUDIT_CODES.NEW);
    await app.clickAndWaitForResponse(
      page.getByTestId("submit-invite-code-button"),
      "validate_invite_code",
    );
    await expect(page.getByTestId("legal-name-input")).toBeVisible();
    await captureState(page, "login-name-step", { stateId: "L14", route: "/login" });
    // ── Error: submit empty name ──
    await expect(page.getByTestId("register-button")).toBeEnabled();
    await page.getByTestId("register-button").click();
    await expect(page.getByTestId("name-error")).toBeVisible();
    await captureElementState(page, "login-name-step", "error", { stateId: "L15" });
  });

  // ── Onboarding ────────────────────────────────────────────────────────────────

  // Participant A uses the long name + long city to exercise text-overflow
  // in cards, badges, and participant-list table cells throughout the rest of
  // the audit flow.
  test("capture onboarding — branch selection + errors", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.selfRegister(AUDIT_PHONES.A, AUDIT_CODES.A, LONG_NAME_A);
    await expect(page).toHaveURL(/\/onboarding/);
    await expect(page.getByTestId("save-onboarding-button")).toBeEnabled();
    await captureState(page, "onboard-branch-selection", { stateId: "O1", route: "/onboarding" });
    // ── Error: np-number = 0 (branch number out of range) ──
    await page.getByTestId("np-city-input").fill(LONG_CITY_A);
    await page.getByTestId("np-number-input").fill("0");
    await app.clickAndWaitForResponse(
      page.getByTestId("save-onboarding-button"),
      "complete_onboarding",
    );
    await expect(page.getByTestId("np-number-error")).not.toBeEmpty();
    await captureElementState(page, "onboard-branch-selection", "error-np-number", { stateId: "O3" });
    // ── Error: city empty (with valid np-number) ──
    // Clear city, set a valid np-number; server should reject empty city.
    await page.getByTestId("np-city-input").fill("");
    await page.getByTestId("np-number-input").fill("1");
    await app.clickAndWaitForResponse(
      page.getByTestId("save-onboarding-button"),
      "complete_onboarding",
    );
    // City error may surface in np-city-error or action-error depending on server routing.
    // We assert on the general save button still being present (not redirected) as the
    // completion signal, then capture whatever error state is shown.
    await expect(page.getByTestId("save-onboarding-button")).toBeVisible();
    await captureElementState(page, "onboard-branch-selection", "error-city", { stateId: "O2" });
    // Complete onboarding with long city so the address carries through the rest
    // of the flow (enrollment forms, participant list, assignment display).
    await app.completeOnboarding(LONG_CITY_A, LONG_BRANCH_A);
  });

  // ── Login: used invite code error (L11) ──────────────────────────────────────

  // Precondition: code A is genuinely consumed — participant A registered with it
  // in the test above. AUDIT_PHONES.NEW never completes registration, so it can
  // revisit the invite step; submitting the consumed code A yields the
  // "already used" error. The rejected submission mutates nothing: code NEW
  // stays unused for the Pass B revoke, and B/C registrations are unaffected.
  test("capture login — used invite code error", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.reachInviteCodeStep(AUDIT_PHONES.NEW);
    await page.getByTestId("invite-code-input").fill(AUDIT_CODES.A);
    await app.clickAndWaitForResponse(
      page.getByTestId("submit-invite-code-button"),
      "validate_invite_code",
    );
    await expect(page.getByTestId("invite-code-error")).not.toBeEmpty();
    await captureElementState(page, "login-invite-code-step", "error-used", { stateId: "L11" });
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
    await captureState(page, "admin-participants-list", { stateId: "A68a", route: "/admin" });
  });

  // ── Home: no season at all ────────────────────────────────────────────────────

  // Inserted between participants list and season creation. At this point no
  // season has been created yet, so participant A sees the NoSeason empty-state.
  test("capture home — no season", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(AUDIT_PHONES.A);
    await app.goHome();
    await expect(page.locator("main")).toBeVisible();
    await captureState(page, "home-no-season", { stateId: "H1", route: "/" });
  });

  // ── Admin: season creation ────────────────────────────────────────────────────

  test("capture admin — no active season (participants list + create form)", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(ADMIN_PHONE);
    await page.goto("/admin");
    await expect(page.getByTestId("create-season-button")).toBeEnabled();
    await captureState(page, "admin-no-season-create-form-available", { stateId: "A3", route: "/admin" });
    // ── Error state: submit create-season with past deadlines ──
    const pastDate = "2020-01-01T00:00";
    await page.getByTestId("signup-deadline-input").fill(pastDate);
    await page.getByTestId("confirm-deadline-input").fill(pastDate);
    await page.getByTestId("create-season-button").click();
    await expect(page.getByTestId("action-error")).not.toBeEmpty();
    await captureElementState(page, "admin-create-season-form", "error");
  });

  test("capture admin — unlaunched season (launch button visible)", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(ADMIN_PHONE);
    await app.createSeason(SIGNUP_DEADLINE, CONFIRM_DEADLINE, SEASON_THEME);
    await expect(page.getByTestId("launch-button")).toBeVisible();
    await captureState(page, "admin-season-created-pre-launch", { stateId: "A8", route: "/admin" });
  });

  // ── Home: no enrollment available (season not yet launched) ──────────────────

  test("capture home — season exists but enrollment not open", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(AUDIT_PHONES.A);
    await app.goHome();
    await captureState(page, "home-enrollment-not-open", { stateId: "H4a", route: "/" });
  });

  // ── Admin: launch → signup phase ─────────────────────────────────────────────

  test("capture admin — signup phase (after launch)", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(ADMIN_PHONE);
    await app.launchSeason();
    await app.goToDashboard();
    await expect(page.getByTestId("advance-button")).toBeVisible();
    await captureState(page, "admin-signup-phase", { stateId: "A7", route: "/admin" });
    // Cancel-confirmation dialog — requires client-side interaction; open, capture, dismiss.
    await expect(page.getByTestId("cancel-button")).toBeEnabled();
    await page.getByTestId("cancel-button").click();
    await expect(page.getByTestId("cancel-confirmation")).toBeVisible();
    await captureSection(page, "admin-signup-phase", "cancel-confirmation");
    await page.getByTestId("cancel-back-button").click();
    await expect(page.getByTestId("cancel-button")).toBeVisible();
  });

  // ── Home: enrollment available ────────────────────────────────────────────────

  test("capture home — enrollment available", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(AUDIT_PHONES.A);
    await app.goHome();
    await expect(page.getByTestId("enroll-button")).toBeVisible();
    await captureState(page, "home-enrollment-available", { stateId: "H2b", route: "/" });
  });

  // ── Home: enrolled state ──────────────────────────────────────────────────────

  test("capture home — enrolled (after enroll)", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(AUDIT_PHONES.A);
    await app.goHome();
    await app.enrollInSeason(LONG_CITY_A, LONG_BRANCH_A);
    await app.expectEnrolled();
    await captureState(page, "home-enrolled", { stateId: "H3", route: "/" });
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
    await captureState(page, "admin-confirm-phase", { stateId: "A27", route: "/admin" });
  });

  // ── Home: confirmed ready ─────────────────────────────────────────────────────

  test("capture home — confirmed ready (after confirm)", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(AUDIT_PHONES.A);
    await app.goHome();
    await expect(page.getByTestId("confirm-ready-button")).toBeVisible();
    await captureState(page, "home-confirm-ready-available", { stateId: "H5", route: "/" });
    await app.confirmReady();
    await app.expectConfirmed();
    await captureState(page, "home-confirmed-ready", { stateId: "H5", route: "/" });
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
    await page.goto("/admin");
    await expect(page.getByTestId("generate-button")).toBeVisible();
    await captureState(page, "admin-assignment-phase-pre-generate", { stateId: "A30", route: "/admin" });
  });

  // ── Home: assigning state ─────────────────────────────────────────────────────

  // Participant sees "organizer preparing assignments" between phase advance
  // and assignment generation.
  test("capture home — assigning state", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(AUDIT_PHONES.A);
    await app.goHome();
    await expect(page.locator("main")).toBeVisible();
    await captureState(page, "home-assigning", { stateId: "H6", route: "/" });
  });

  // ── Admin: assignments generated ─────────────────────────────────────────────

  test("capture admin — assignment generated (cycle visualization)", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(ADMIN_PHONE);
    await app.generateAssignments();
    await expect(page.getByTestId("cycle-visualization")).toBeVisible();
    await captureState(page, "admin-assignment-cycle", { stateId: "A36", route: "/admin" });
    await captureSection(page, "admin-assignment-cycle", "cycle-visualization");
  });

  // ── Admin: swap form error ────────────────────────────────────────────────────

  test("capture admin — swap form error (same participant both slots)", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(ADMIN_PHONE);
    await app.goToDashboard();
    await expect(page.getByTestId("override-available")).toBeVisible();
    // Select the same participant for both sender slots to trigger a server error.
    const senderASelect = page.getByTestId("sender-a-input");
    const senderBSelect = page.getByTestId("sender-b-input");
    await expect(senderASelect).toBeVisible();
    await expect(senderBSelect).toBeVisible();
    // Pick the first option value from sender A, set same in sender B.
    const firstValue = await senderASelect.locator("option").nth(1).getAttribute("value");
    if (firstValue) {
      await senderASelect.selectOption(firstValue);
      await senderBSelect.selectOption(firstValue);
    }
    await expect(page.getByTestId("swap-button")).toBeEnabled();
    await app.clickAndWaitForResponse(
      page.getByTestId("swap-button"),
      "swap_assignments",
    );
    await expect(page.getByTestId("action-error")).toBeVisible();
    await captureElementState(page, "admin-swap-form", "error", { stateId: "A40" });
  });

  // ── Phase: advance to delivery ────────────────────────────────────────────────

  test("capture admin — delivery phase (SMS controls)", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(ADMIN_PHONE);
    await app.advanceSeason();
    await page.goto("/admin");
    await expect(page.getByTestId("send-assignment-button")).toBeVisible();
    await captureState(page, "admin-delivery-phase-sms-controls", { stateId: "A41", route: "/admin" });
  });

  // ── Admin: SMS report after sending ──────────────────────────────────────────

  test("capture admin — SMS report (after assignment SMS)", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(ADMIN_PHONE);
    await app.triggerSms("assignment");
    await expect(page.getByTestId("sms-report")).toBeVisible();
    await captureState(page, "admin-sms-report", { stateId: "A42", route: "/admin" });
    await captureSection(page, "admin-sms-report", "sms-report");
  });

  // ── Home: participant sees assignment ─────────────────────────────────────────

  test("capture home — assignment visible to participant", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(AUDIT_PHONES.A);
    await app.goHome();
    await expect(page.getByTestId("recipient-name")).toBeVisible();
    await expect(page.getByTestId("received-button")).toBeVisible();
    await captureState(page, "home-assignment-and-receipt-form", { stateId: "H7", route: "/" });
  });

  // ── Home: receipt confirmed (thank you) ──────────────────────────────────────

  test("capture home — receipt confirmed (thank you)", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(AUDIT_PHONES.A);
    await app.goHome();
    await app.confirmReceipt(true);
    await expect(page.getByTestId("receipt-thanks")).toBeVisible();
    await captureState(page, "home-receipt-confirmed-thanks", { stateId: "H8", route: "/" });
  });

  // ── Home: not-received receipt ────────────────────────────────────────────────

  // Participant B reports NOT received before the season advances to complete.
  // A and C confirm received. The "not received" state is the visual target.
  // Note: if the server requires all receipts to advance to complete, B's
  // not-received report is captured but the season-complete capture below is
  // skipped gracefully (B's state is still captured either way).
  test("capture home — not-received receipt (participant B)", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(AUDIT_PHONES.B);
    await app.goHome();
    await expect(page.getByTestId("received-button")).toBeVisible();
    await app.confirmReceipt(false);
    // Capture the result — POM waits for "reported" text or not-received indicator.
    await captureState(page, "home-receipt-not-received", { stateId: "H7", route: "/" });
  });

  // ── Setup: C confirms receipt (B remains not-received) ───────────────────────

  test("setup — participant C confirms receipt", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(AUDIT_PHONES.C);
    await app.goHome();
    await app.confirmReceipt(true);
  });

  // ── Phase: advance to complete ────────────────────────────────────────────────

  // If B's not-received status blocks advance, this test (and subsequent ones
  // in this serial block) will fail. That is acceptable — the not-received
  // capture is already obtained.
  test("capture admin — complete phase", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(ADMIN_PHONE);
    await app.advanceSeason();
    await page.goto("/admin");
    await expect(page.locator("main")).toBeVisible();
    await captureState(page, "admin-season-complete", { stateId: "A47", route: "/admin" });
  });

  // ── Home: season complete ─────────────────────────────────────────────────────

  test("capture home — season complete", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(AUDIT_PHONES.A);
    await app.goHome();
    await app.expectHomeContent(/complete|завершено/i);
    await captureState(page, "home-season-complete", { stateId: "H9", route: "/" });
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
    await captureState(page, "admin-season-cancelled", { stateId: "A52", route: "/admin" });
  });

  test("capture home — cancelled season state", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(AUDIT_PHONES.A);
    await app.goHome();
    await expect(page.locator("main")).toBeVisible();
    await captureState(page, "home-cancelled-season", { stateId: "H10", route: "/" });
  });

  // ── Pass B: post-lifecycle mutations ─────────────────────────────────────────
  // After the full lifecycle + cancel, the DB has:
  //   - 4 invite codes: A/B/C used, NEW unused
  //   - 3 active participants
  // We mutate to capture revoked/inactive states.

  test("capture admin — mixed invite code statuses (after revoke)", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(ADMIN_PHONE);
    // Revoke code NEW — it was never consumed (participant NEW never finished registration).
    await app.revokeInviteCode(AUDIT_CODES.NEW);
    await app.goToDashboard();
    await expect(page.getByTestId("invite-code-list")).toBeVisible();
    // At this point: A/B/C = used (gray), NEW = revoked (gray), no unused.
    await captureState(page, "admin-invite-codes-mixed-statuses", { stateId: "A61b+A61e", route: "/admin" });
  });

  test("capture admin — mixed participant statuses (after deactivate)", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(ADMIN_PHONE);
    // Deactivate participant C — shows active/inactive mix in the list.
    await app.deactivateParticipant(AUDIT_NAMES.C);
    await app.goToDashboard();
    await expect(page.getByTestId("participant-list")).toBeVisible();
    await captureState(page, "admin-participants-mixed-statuses", { stateId: "A68b", route: "/admin" });
  });

  // ── Pass C: existing-address enrollment ──────────────────────────────────────
  // Create a third season. Participant A already has an address from the first
  // enrollment. The EnrollmentOpen state shows the existing-address branch
  // (info-list display, hidden inputs, no text fields visible).

  test("setup — create and launch third season for existing-address capture", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(ADMIN_PHONE);
    await app.createSeason(futureDatetime(7), futureDatetime(21));
    await app.launchSeason();
  });

  test("capture home — enrollment open with existing address", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(AUDIT_PHONES.A);
    await app.goHome();
    // Participant A has an address; the EnrollmentOpen state shows the
    // existing-address branch with the info-list and hidden inputs.
    await expect(page.getByTestId("enroll-button")).toBeVisible();
    await captureState(page, "home-enrollment-existing-address", { stateId: "H2a", route: "/" });
  });

});
