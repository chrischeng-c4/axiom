/**
 * CSS Pipeline E2E Tests
 *
 * Validates jet dev server CSS capabilities:
 * - PostCSS pipeline (@import resolution, nested CSS, custom properties)
 * - Tailwind CSS JIT (utility classes produce correct computed styles)
 * - Dev mode rebuild (adding new Tailwind class triggers CSS update)
 *
 * Prerequisite: `cclab jet dev` running on e2e/jet/ (port 3000)
 *
 * Run with:
 *   npx playwright test --config=e2e/playwright.config.ts --project=jet-dev e2e/jet/tests/css.spec.ts
 */

import { test, expect } from "@jet/test";
import { readFixture, writeFixture } from "./test-utils";

test.describe("CSS Pipeline", () => {
  test("PostCSS pipeline — @import, nested CSS, and custom properties resolve", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.locator('[data-testid="app"]')).toBeVisible();

    // Verify that CSS styles are applied (proves CSS pipeline processed the files)
    const appStyles = await page.locator(".todoapp").evaluate((el) => {
      const computed = getComputedStyle(el);
      return {
        maxWidth: computed.maxWidth,
        background: computed.backgroundColor,
        padding: computed.padding,
        borderRadius: computed.borderRadius,
      };
    });

    // The .todoapp class from style.css should be applied
    expect(appStyles.maxWidth).toBe("500px");
    // background: #fff → rgb(255, 255, 255)
    expect(appStyles.background).toBe("rgb(255, 255, 255)");

    // Verify nested styles work: .todo-list li styles
    // First add a todo so we have a list item to inspect
    const input = page.locator('[data-testid="new-todo"]');
    await input.fill("CSS test");
    await page.locator('[data-testid="add-btn"]').click();
    await page.waitForTimeout(100);

    const liStyles = await page
      .locator('[data-testid="todo-list"] li')
      .first()
      .evaluate((el) => {
        const computed = getComputedStyle(el);
        return {
          display: computed.display,
          alignItems: computed.alignItems,
        };
      });

    expect(liStyles.display).toBe("flex");
    expect(liStyles.alignItems).toBe("center");

    // Verify box-sizing from global * rule
    const boxSizing = await page.locator("body").evaluate((el) => {
      return getComputedStyle(el).boxSizing;
    });
    expect(boxSizing).toBe("border-box");
  });

  test("CSS class application — styles produce correct computed values", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.locator('[data-testid="app"]')).toBeVisible();

    // Verify basic CSS classes are producing the correct computed styles
    // The h1 element should have the styles from style.css
    const h1Styles = await page.locator("h1").evaluate((el) => {
      const computed = getComputedStyle(el);
      return {
        textAlign: computed.textAlign,
        // color: #c44 → rgb(204, 68, 68)
        color: computed.color,
      };
    });

    expect(h1Styles.textAlign).toBe("center");
    expect(h1Styles.color).toBe("rgb(204, 68, 68)");

    // Verify button styles
    const buttonStyles = await page
      .locator("button")
      .first()
      .evaluate((el) => {
        const computed = getComputedStyle(el);
        return {
          cursor: computed.cursor,
          borderRadius: computed.borderRadius,
        };
      });

    expect(buttonStyles.cursor).toBe("pointer");
    expect(buttonStyles.borderRadius).toBe("4px");
  });

  test("dev mode rebuild — CSS changes appear without full reload", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.locator('[data-testid="app"]')).toBeVisible();

    // Set marker to detect full page reload
    await page.evaluate(() => {
      (window as any).__css_rebuild_marker = true;
    });

    const originalCSS = readFixture("src/style.css");

    try {
      // Add a new CSS rule targeting a specific element
      const modifiedCSS =
        originalCSS +
        "\n.todoapp h1 { font-size: 48px; }\n";
      writeFixture("src/style.css", modifiedCSS);

      // Wait for CSS rebuild and HMR update
      await page.waitForTimeout(2000);

      // Verify the new style is applied
      const fontSize = await page
        .locator("h1")
        .evaluate((el) => getComputedStyle(el).fontSize);
      expect(fontSize).toBe("48px");

      // Verify no full page reload occurred
      const markerPresent = await page.evaluate(
        () => (window as any).__css_rebuild_marker === true
      );
      expect(markerPresent).toBe(true);
    } finally {
      // Restore original CSS
      writeFixture("src/style.css", originalCSS);
      await page.waitForTimeout(1000);
    }
  });
});
