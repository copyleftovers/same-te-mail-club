import { test as base } from "@playwright/test";
import * as fs from "fs";
import * as path from "path";
import * as os from "os";

/**
 * Static asset cache for Playwright tests.
 *
 * Each `npx playwright test` invocation creates a shared temp directory.
 * Route interception caches GET responses for .wasm, .js, .css, and .woff2
 * files on first download, then serves from disk on subsequent tests.
 *
 * This eliminates ~57 redundant downloads of the 14MB dev WASM bundle,
 * reducing total bytes served from ~812MB to ~14MB per E2E run.
 *
 * Cache key: URL pathname (not full URL, so baseURL changes don't invalidate).
 * Only GET requests for static file extensions are cached.
 * HTML and API responses are never cached.
 */

const CACHEABLE_EXTENSIONS = [".wasm", ".js", ".css", ".woff2"];

// Single temp directory shared across all tests in this process.
// Created lazily on first use. Cleaned up by the OS (temp dir).
let cacheDir: string | null = null;

function getCacheDir(): string {
  if (!cacheDir) {
    cacheDir = fs.mkdtempSync(path.join(os.tmpdir(), "pw-asset-cache-"));
  }
  return cacheDir;
}

function getCachePath(urlPath: string): string {
  // Replace path separators to create a flat cache directory.
  // "/pkg/samete.wasm" -> "pkg__samete.wasm"
  const sanitized = urlPath.replace(/^\//, "").replace(/\//g, "__");
  return path.join(getCacheDir(), sanitized);
}

function isCacheableUrl(url: string): boolean {
  try {
    const parsed = new URL(url);
    return CACHEABLE_EXTENSIONS.some((ext) =>
      parsed.pathname.endsWith(ext),
    );
  } catch {
    return false;
  }
}

export const test = base.extend({
  page: async ({ page }, use) => {
    await page.route(
      (url) => isCacheableUrl(url.toString()),
      async (route) => {
        const request = route.request();

        // Only cache GET requests.
        if (request.method() !== "GET") {
          await route.continue();
          return;
        }

        const url = new URL(request.url());
        const cachePath = getCachePath(url.pathname);
        const metaPath = cachePath + ".meta.json";

        // Serve from cache if available.
        // Use `path` instead of `body` — Playwright streams from disk
        // rather than serializing through CDP (avoids base64-encoding
        // 14MB WASM files through the protocol pipe).
        if (fs.existsSync(cachePath) && fs.existsSync(metaPath)) {
          const meta = JSON.parse(fs.readFileSync(metaPath, "utf-8"));
          await route.fulfill({
            status: meta.status,
            headers: meta.headers,
            path: cachePath,
          });
          return;
        }

        // Fetch from server, cache, and fulfill.
        const response = await route.fetch();
        const body = await response.body();
        const status = response.status();
        const headers = response.headers();

        // Only cache successful responses.
        if (status >= 200 && status < 300) {
          fs.writeFileSync(cachePath, body);
          fs.writeFileSync(
            metaPath,
            JSON.stringify({ status, headers }),
          );
        }

        // First request: fulfill with body directly (already in memory).
        // Subsequent requests hit the cache path above.
        await route.fulfill({
          status,
          headers,
          body,
        });
      },
    );

    await use(page);
  },
});

export { expect } from "@playwright/test";
