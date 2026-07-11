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
 *
 * INDEX.md merging: the main visual-audit.spec.ts afterAll rewrites INDEX.md from
 * scratch (it owns the MANIFEST array). The cohort pass runs after and APPENDS its
 * rows. Idempotency: before appending, existing rows for cohort files are removed so
 * a re-run never duplicates entries. The main pass afterAll never sees cohort files
 * (different screenshot paths), so ordering is safe regardless of which runs first.
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

// The five files this spec produces (relative to screenshots/).
// Kept in sync with the actual screenshot() calls below so the INDEX merge
// can locate them without parsing the raw paths a second time.
const COHORT_FILES = [
  "light-desktop/admin-cycle-viz-12-nodes.png",
  "light-mobile/admin-cycle-viz-12-nodes.png",
  "dark-desktop/admin-cycle-viz-12-nodes.png",
  "dark-mobile/admin-cycle-viz-12-nodes.png",
  "sections/admin-cycle-viz-12-nodes__cycle-visualization.png",
] as const;

const COHORT_STATE_ID = "A36-cohort";
const COHORT_ROUTE = "/admin";

// Opt-in gate: the default suite (`npx playwright test`, no filter) discovers
// every spec under testDir — without this gate `just e2e` would fire the cohort
// seed against the shared `samete` DB. When every test in the file is skipped,
// Playwright also skips beforeAll, so the seed never executes.
test.skip(
  process.env.COHORT_CAPTURE !== "1",
  "cohort capture is opt-in — the harness exports COHORT_CAPTURE=1 for the cohort invocation",
);

// Defense in depth: refuse to seed the shared dev DB even if the gate is
// bypassed. Mirrors the harness's own `samete` guard.
function assertSiblingDatabaseUrl(rawUrl: string | undefined): string {
  if (!rawUrl) {
    throw new Error(
      "DATABASE_URL is not set — the cohort seed requires the harness-injected sibling DB URL",
    );
  }
  const dbName = new URL(rawUrl).pathname.replace(/^\//, "");
  if (dbName === "samete") {
    throw new Error(
      "DATABASE_URL points at the shared 'samete' DB — refusing to seed; cohort pass must target a samete_<suffix> sibling",
    );
  }
  return rawUrl;
}

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
  const databaseUrl = assertSiblingDatabaseUrl(process.env.DATABASE_URL);
  const seedFile = path.resolve(__dirname, "fixtures/cohort-seed.sql");
  execSync(`psql "${databaseUrl}" -f "${seedFile}"`, {
    stdio: "inherit",
  });
});

// ── INDEX.md merge ─────────────────────────────────────────────────────────────
//
// The main visual-audit.spec.ts afterAll rewrites screenshots/INDEX.md from
// scratch for every run. This cohort afterAll appends the cohort rows so the
// INDEX ends complete after the standard two-invocation sequence (main → cohort).
//
// Idempotency: existing rows whose `file` column matches a COHORT_FILES entry
// are stripped before the new rows are appended. A re-run therefore never
// duplicates entries. The main pass afterAll owns its own MANIFEST array and
// never references cohort paths, so the ordering is safe if cohort runs first
// (the main pass would later overwrite without cohort rows — that case is a
// partial run and is expected to produce a partial INDEX).

test.afterAll(() => {
  const indexPath = "screenshots/INDEX.md";

  // Read the existing INDEX (written by the main pass) or start fresh.
  let existing = "";
  try {
    existing = fs.readFileSync(indexPath, "utf8");
  } catch {
    // INDEX does not exist yet — cohort ran before main pass; start from scratch.
    existing = [
      "# Screenshot Index",
      "Generated: (cohort-only run)",
      "",
      "| File | State | Route |",
      "|------|-------|-------|",
      "",
    ].join("\n");
  }

  // Strip any previously-appended cohort rows so re-runs don't duplicate.
  const lines = existing.split("\n");
  const withoutCohortRows = lines.filter(
    (line) => !COHORT_FILES.some((f) => line.includes(f)),
  );

  // Append the fresh cohort rows.
  const cohortRows = COHORT_FILES.map(
    (f) => `| ${f} | ${COHORT_STATE_ID} | ${COHORT_ROUTE} |`,
  );

  // Ensure there is exactly one trailing newline before appending.
  const base = withoutCohortRows.join("\n").trimEnd();
  const merged = [base, ...cohortRows, ""].join("\n");

  fs.writeFileSync(indexPath, merged);
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
