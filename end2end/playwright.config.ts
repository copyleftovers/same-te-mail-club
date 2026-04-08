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
  timeout: 60_000,
  expect: { timeout: 15_000 },

  // Sequential execution — tests share database state.
  fullyParallel: false,
  workers: 1,

  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,

  // Both reporters simultaneously: list for terminal visibility, html for failure analysis.
  reporter: process.env.CI
    ? [["list"], ["html", { open: "never" }], ["json", { outputFile: "results.json" }]]
    : [["list"], ["html", { open: "on-failure" }], ["json", { outputFile: "results.json" }]],

  use: {
    baseURL: "http://127.0.0.1:3000",
    trace: "on-first-retry",
    screenshot: "only-on-failure",

    // Prevent individual actions from hanging on hydration/network issues.
    actionTimeout: 10_000,
    // Dev WASM bundle is ~14MB; SSR can slow under sustained load.
    // 30s gives headroom without masking real failures (passing tests are ~3s).
    navigationTimeout: 30_000,
  },

  // Single browser — chromium only. Multi-browser is not worth the cost at this scale.
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
});
