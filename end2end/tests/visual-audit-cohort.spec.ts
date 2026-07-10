/**
 * Unit 7 — Large-cohort cycle-viz capture pass.
 *
 * Captures the admin cycle-visualization SVG rendered with 12 participants
 * (double-barrel Ukrainian names, single ring) in all four
 * screenshot matrix slots: light-desktop, light-mobile, dark-desktop, dark-mobile,
 * plus one section crop of the SVG element itself.
 *
 * Invocation (from the worktree root):
 *   VISUAL_SPEC=tests/visual-audit-cohort.spec.ts bash scripts/isolated-capture.sh cohort visual
 *
 * The harness seeds test_admin.sql before running. This spec's beforeAll seeds
 * the 12-participant cohort via cohort-seed.sql using DATABASE_URL from the harness env.
 */

import { execSync } from "child_process";
import * as fs from "fs";
import * as path from "path";
import { test, expect } from "./fixtures/cached-context";
import { MailClubPage } from "./fixtures/mail_club_page";

// Admin phone matches seed/test_admin.sql
const ADMIN_PHONE = "+380670000001";

const MOBILE_VIEWPORT  = { width: 375, height: 812 } as const;
const DESKTOP_VIEWPORT = { width: 1280, height: 800 } as const;

// Paint-settle delay reused from visual-audit.spec.ts precedent.
// Exempted from waitForTimeout ban: this is a capture harness paint settle,
// not a test-assertion wait (see deferred_items: `waitForTimeout(LAYOUT_REFLOW_MS)`).
const LAYOUT_REFLOW_MS = 150;

test.beforeAll(() => {
  // Create screenshot directories (mirrors visual-audit.spec.ts structure).
  for (const dir of [
    "screenshots/light-desktop",
    "screenshots/light-mobile",
    "screenshots/dark-desktop",
    "screenshots/dark-mobile",
    "screenshots/sections",
  ]) {
    fs.mkdirSync(dir, { recursive: true });
  }

  // Seed the 12-participant cohort into the harness's sibling DB.
  // DATABASE_URL is injected by isolated-capture.sh; it points at samete_<suffix>,
  // never at the dev samete DB.
  const seedFile = path.resolve(__dirname, "fixtures/cohort-seed.sql");
  execSync(`psql "${process.env.DATABASE_URL}" -f "${seedFile}"`, {
    stdio: "inherit",
  });
});

test.describe.serial("Cohort Capture", () => {
  test("admin — 12-node cycle visualization", async ({ page }) => {
    const app = new MailClubPage(page);

    // Login as admin, then navigate explicitly (conventions.md: goto after login is required).
    await app.login(ADMIN_PHONE);
    await page.goto("/admin");

    // Wait for cycle-visualization to be present — confirms the assignment data rendered.
    await expect(page.getByTestId("cycle-visualization")).toBeVisible();

    // Capture full-page in all four matrix slots.
    for (const scheme of ["light", "dark"] as const) {
      await page.emulateMedia({ colorScheme: scheme });
      await page.waitForTimeout(LAYOUT_REFLOW_MS);

      for (const [viewport, label] of [
        [DESKTOP_VIEWPORT, "desktop"],
        [MOBILE_VIEWPORT, "mobile"],
      ] as const) {
        await page.setViewportSize(viewport);
        await page.waitForTimeout(LAYOUT_REFLOW_MS);
        await page.screenshot({
          path: `screenshots/${scheme}-${label}/admin-cycle-viz-12-nodes.png`,
          fullPage: true,
        });
      }
    }

    // Restore to light + desktop for the section crop.
    await page.emulateMedia({ colorScheme: "light" });
    await page.setViewportSize(DESKTOP_VIEWPORT);
    await page.waitForTimeout(LAYOUT_REFLOW_MS);

    // Section crop: the SVG element alone — too small to evaluate in the full-page shot.
    await page.getByTestId("cycle-visualization").first().screenshot({
      path: "screenshots/sections/admin-cycle-viz-12-nodes__cycle-visualization.png",
    });
  });
});
