import { defineConfig, devices } from "@playwright/test";

/**
 * Playwright config for The Mail Club E2E tests.
 *
 * cargo-leptos manages the server lifecycle:
 *   cargo leptos end-to-end
 *
 * The webServer block is intentionally absent — cargo-leptos starts
 * and stops the server around test execution.
 */
export default defineConfig({
  testDir: "./tests",
  timeout: 30_000,
  expect: { timeout: 5_000 },

  // Sequential execution — tests share database state
  fullyParallel: false,
  workers: 1,

  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? "list" : "html",

  use: {
    baseURL: "http://127.0.0.1:3000",
    trace: "on-first-retry",
    screenshot: "only-on-failure",
  },

  // Single browser — chromium only. Multi-browser is not worth the cost at this scale.
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
});
