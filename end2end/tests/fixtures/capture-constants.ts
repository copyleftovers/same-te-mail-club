import { type Page } from "@playwright/test";

/**
 * Shared capture-harness constants for the visual-audit and E2E test suites.
 *
 * These values change together (e.g. a different admin seed phone requires one
 * edit here, not three). Extract point: three specs declared them independently;
 * silent drift between them is the failure mode this file eliminates.
 */

// ── Seeded admin phone ─────────────────────────────────────────────────────────
// Matches the phone inserted by `justfile db-seed` (test_admin.sql).
// All specs that login as admin reference this one value.

export const ADMIN_PHONE = "+380670000001";

// ── Viewport dimensions ────────────────────────────────────────────────────────
// Used by the capture harness to shoot each state at both sizes.
// Values match the canonical mobile / desktop sizes established in the
// visual-audit spec and carried forward to the cohort pass.

export const MOBILE_VIEWPORT = { width: 375, height: 812 } as const;
export const DESKTOP_VIEWPORT = { width: 1280, height: 800 } as const;

// ── Paint-settle delay ─────────────────────────────────────────────────────────
// Documented exception to the waitForTimeout ban: used only in capture helpers
// as a paint-timing settle before taking screenshots, never as an assertion wait.
// See deferred_items: `waitForTimeout(LAYOUT_REFLOW_MS)`.

export const LAYOUT_REFLOW_MS = 150;

/**
 * Wait for CSS layout and paint to settle before taking a screenshot.
 *
 * This is the ONLY sanctioned use of `waitForTimeout` in the test suite.
 * It belongs in capture helpers, not in assertions. Calling it elsewhere
 * violates the banned-practices section of end2end/README.md.
 */
export async function paintSettle(page: Page): Promise<void> {
  await page.waitForTimeout(LAYOUT_REFLOW_MS);
}

// ── Deadline helper ────────────────────────────────────────────────────────────
// Generates a future datetime string for season signup/confirm deadlines.
// SAMETE_TEST_MODE=true bypasses deadline gates, so exact values matter only
// for the form field — the value just needs to be parseable and in the future.

/**
 * Returns a datetime-local string (YYYY-MM-DDTHH:mm) for a date that many
 * days from now. Use this wherever a season deadline input needs a future date.
 */
export function futureDeadline(daysFromNow: number): string {
  const d = new Date(Date.now() + daysFromNow * 24 * 60 * 60 * 1000);
  return d.toISOString().slice(0, 16);
}
