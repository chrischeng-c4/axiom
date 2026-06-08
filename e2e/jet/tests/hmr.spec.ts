/**
 * HMR (Hot Module Replacement) E2E Tests
 *
 * Validates jet dev server HMR capabilities:
 * - Module HMR update (component re-renders without full reload)
 * - React Fast Refresh (component state preserved after edit)
 * - CSS HMR (style updates without page reload)
 * - Error overlay (syntax error shows overlay, fix dismisses it)
 *
 * Mechanism: Set state markers via page.evaluate(), modify source files
 * via Node.js fs, observe DOM changes and WebSocket messages on /__jet_hmr.
 *
 * Prerequisite: `cclab jet dev` running on e2e/jet/ (port 3000)
 *
 * Run with:
 *   npx playwright test --config=e2e/playwright.config.ts --project=jet-dev e2e/jet/tests/hmr.spec.ts
 */

import { test, expect } from "@jet/test";
import { readFixture, writeFixture, addTodo } from "./test-utils";

test.describe("Jet HMR", () => {
  test("module HMR update — component re-renders without full reload", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.locator('[data-testid="app"]')).toBeVisible();

    // Set a marker on window to detect full page reload
    await page.evaluate(() => {
      (window as any).__hmr_marker = true;
    });

    // Read original TodoItem source
    const originalSource = readFixture("src/components/TodoItem.tsx");

    // Add a todo so TodoItem renders
    await addTodo(page, "HMR reload test");

    try {
      // Modify TodoItem: add a data attribute to detect the update
      const modifiedSource = originalSource.replace(
        'className="todo-text"',
        'className="todo-text" data-hmr-updated="true"'
      );
      writeFixture("src/components/TodoItem.tsx", modifiedSource);

      // Wait for HMR to process
      await page.waitForTimeout(2000);

      // Page should NOT have fully reloaded (marker still present)
      const markerPresent = await page.evaluate(
        () => (window as any).__hmr_marker === true
      );
      expect(markerPresent).toBe(true);

      // DOM should reflect the HMR update
      await expect(
        page.locator('[data-hmr-updated="true"]')
      ).toBeVisible();
    } finally {
      // Restore original source
      writeFixture("src/components/TodoItem.tsx", originalSource);
      await page.waitForTimeout(1000);
    }
  });

  test("React Fast Refresh — component state preserved after edit", async ({
    page,
  }) => {
    await page.goto("/");

    // Add a todo to create state
    await addTodo(page, "HMR test item");
    const items = page.locator('[data-testid="todo-list"] li');
    await expect(items).toHaveCount(1);

    const originalSource = readFixture("src/components/TodoItem.tsx");

    try {
      // Modify TodoItem component (non-breaking change)
      const modifiedSource = originalSource.replace(
        'className="destroy"',
        'className="destroy" data-fast-refresh="true"'
      );
      writeFixture("src/components/TodoItem.tsx", modifiedSource);

      // Wait for Fast Refresh to process
      await page.waitForTimeout(2000);

      // State should be preserved: todo item still in the list
      await expect(items).toHaveCount(1);
      const todoText = items.first().locator(".todo-text");
      await expect(todoText).toHaveText("HMR test item");
    } finally {
      // Restore original source
      writeFixture("src/components/TodoItem.tsx", originalSource);
      await page.waitForTimeout(1000);
    }
  });

  test("CSS HMR — styles update without page reload", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator('[data-testid="app"]')).toBeVisible();

    // Set reload detection marker
    await page.evaluate(() => {
      (window as any).__css_hmr_marker = true;
    });

    const originalCSS = readFixture("src/style.css");

    try {
      // Modify CSS: change the todoapp background color
      const modifiedCSS = originalCSS.replace(
        "background: #fff;",
        "background: rgb(255, 240, 240);"
      );
      writeFixture("src/style.css", modifiedCSS);

      // Wait for CSS HMR to process
      await page.waitForTimeout(2000);

      // Check that the style updated
      const bgColor = await page
        .locator(".todoapp")
        .evaluate((el) => getComputedStyle(el).backgroundColor);
      expect(bgColor).toBe("rgb(255, 240, 240)");

      // No full page reload
      const markerPresent = await page.evaluate(
        () => (window as any).__css_hmr_marker === true
      );
      expect(markerPresent).toBe(true);
    } finally {
      // Restore original CSS
      writeFixture("src/style.css", originalCSS);
      await page.waitForTimeout(1000);
    }
  });

  test("error overlay — syntax error shows overlay, fix dismisses it", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.locator('[data-testid="app"]')).toBeVisible();

    const originalSource = readFixture("src/app.tsx");

    try {
      // Introduce a syntax error
      const brokenSource = originalSource.replace(
        "export function App() {",
        "export function App() { const x = <<<SYNTAX_ERROR>>>;"
      );
      writeFixture("src/app.tsx", brokenSource);

      // Wait for error overlay to appear
      await page.waitForTimeout(2000);

      // Error overlay should be visible (check common overlay selectors)
      const hasOverlay = await page.evaluate(() => {
        // Jet error overlay may use a specific element or shadow DOM
        const overlay =
          document.querySelector("[data-jet-error-overlay]") ||
          document.querySelector(".jet-error-overlay") ||
          document.querySelector("vite-error-overlay");
        return overlay !== null;
      });
      expect(hasOverlay).toBe(true);

      // Fix the syntax error by restoring original source
      writeFixture("src/app.tsx", originalSource);

      // Wait for overlay to dismiss
      await page.waitForTimeout(2000);

      // Overlay should be gone and app should be functional
      const overlayGone = await page.evaluate(() => {
        const overlay =
          document.querySelector("[data-jet-error-overlay]") ||
          document.querySelector(".jet-error-overlay") ||
          document.querySelector("vite-error-overlay");
        return overlay === null;
      });
      expect(overlayGone).toBe(true);

      // App should be functional again
      await expect(page.locator('[data-testid="app"]')).toBeVisible();
    } finally {
      // Ensure original source is restored even if test fails
      writeFixture("src/app.tsx", originalSource);
      await page.waitForTimeout(1000);
    }
  });
});
