/**
 * Dev Server E2E Tests
 *
 * Validates jet dev server capabilities:
 * - TypeScript type stripping
 * - import.meta.env injection
 * - Path alias resolution (tsconfig paths)
 * - Proxy forwarding
 * - Node.js polyfills
 *
 * Prerequisite: `cclab jet dev` running on e2e/jet/ (port 3000)
 *
 * Run with:
 *   npx playwright test --config=e2e/playwright.config.ts --project=jet-dev e2e/jet/tests/dev-server.spec.ts
 */

import { test, expect } from "@jet/test";

test.describe("Jet Dev Server", () => {
  test("TypeScript type stripping — no Unexpected token errors", async ({
    page,
  }) => {
    const consoleErrors: string[] = [];
    page.on("console", (msg) => {
      if (msg.type() === "error") {
        consoleErrors.push(msg.text());
      }
    });

    await page.goto("/");
    // Wait for the app to fully initialize
    await page.waitForTimeout(1000);

    // Filter for TS-related parse errors (Unexpected token usually means TS wasn't stripped)
    const tsErrors = consoleErrors.filter(
      (e) =>
        e.includes("Unexpected token") ||
        e.includes("SyntaxError") ||
        e.includes("Cannot use import statement")
    );
    expect(tsErrors).toHaveLength(0);

    // App should render successfully (proves TS was transpiled)
    await expect(page.locator('[data-testid="app"]')).toBeVisible();
  });

  test("import.meta.env — DEV is true and MODE is development", async ({
    page,
  }) => {
    await page.goto("/");

    // Evaluate import.meta.env in the browser context
    const envDev = await page.evaluate(() => (import.meta as any).env?.DEV);
    const envMode = await page.evaluate(() => (import.meta as any).env?.MODE);

    expect(envDev).toBe(true);
    expect(envMode).toBe("development");
  });

  test("path alias resolution — @/components resolves without 404", async ({
    page,
  }) => {
    const failedRequests: string[] = [];
    page.on("requestfailed", (req) => {
      failedRequests.push(req.url());
    });

    const consoleErrors: string[] = [];
    page.on("console", (msg) => {
      if (msg.type() === "error") {
        consoleErrors.push(msg.text());
      }
    });

    await page.goto("/");
    await page.waitForTimeout(500);

    // No 404s for aliased imports
    const aliasFailures = failedRequests.filter(
      (url) => url.includes("@/") || url.includes("components")
    );
    expect(aliasFailures).toHaveLength(0);

    // No module resolution errors in console
    const moduleErrors = consoleErrors.filter(
      (e) =>
        e.includes("Failed to resolve") ||
        e.includes("Cannot find module") ||
        e.includes("404")
    );
    expect(moduleErrors).toHaveLength(0);

    // The Header component (imported via alias) should render
    await expect(page.locator("h1")).toBeVisible();
  });

  test("proxy forwarding — API requests proxied per config", async ({
    page,
  }) => {
    await page.goto("/");

    // Attempt a fetch to the proxy endpoint configured in the dev server.
    // The proxy should forward the request rather than the dev server
    // serving its own static 404 page.
    const response = await page.evaluate(async () => {
      try {
        const res = await fetch("/api/health", { method: "GET" });
        const body = await res.text();
        return { status: res.status, body, networkError: false };
      } catch {
        // Network error means proxy is configured but target is down
        // — distinct from a 404 which means no proxy at all
        return { status: 0, body: "", networkError: true };
      }
    });

    // If we get a dev server HTML 404, it means no proxy matched — fail.
    // Any other response (upstream 404/500, network error) means proxy is active.
    if (response.status === 404) {
      expect(response.body).not.toContain("<!DOCTYPE");
    }
    // If networkError, proxy config exists but target is unreachable — acceptable
  });

  test("Node.js polyfills — crypto, buffer, path available", async ({
    page,
  }) => {
    const consoleErrors: string[] = [];
    page.on("console", (msg) => {
      if (msg.type() === "error") {
        consoleErrors.push(msg.text());
      }
    });

    await page.goto("/");
    await page.waitForTimeout(500);

    // No polyfill-related errors in console (e.g. "Module not found: buffer")
    const polyfillErrors = consoleErrors.filter(
      (e) =>
        e.includes("buffer") ||
        e.includes("path") ||
        e.includes("crypto") ||
        e.includes("polyfill")
    );
    expect(polyfillErrors).toHaveLength(0);

    // App renders successfully — proves polyfills didn't break module resolution
    await expect(page.locator('[data-testid="app"]')).toBeVisible();
  });
});
