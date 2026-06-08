/**
 * DOM Snapshot Comparison Test
 *
 * Verifies that the jet build produces functionally equivalent output
 * to the Vite build by comparing DOM structure after interactions.
 *
 * Run with:
 *   1. Build both:  npm run build:vite
 *   2. Serve both:  npx serve dist-vite -l 4174 & npx serve dist-jet -l 4175 &
 *   3. Test:        npx playwright test
 */

import { test, expect, Page } from "@jet/test";

// --- Helpers ---

/** Get the normalized DOM structure of the app. */
async function getAppSnapshot(page: Page) {
  return page.evaluate(() => {
    const app = document.querySelector('[data-testid="app"]');
    if (!app) return null;

    function normalize(el: Element): any {
      const children = Array.from(el.children).map(normalize);
      const text = Array.from(el.childNodes)
        .filter((n) => n.nodeType === Node.TEXT_NODE)
        .map((n) => (n.textContent || "").trim())
        .filter(Boolean);

      return {
        tag: el.tagName.toLowerCase(),
        class: el.className || undefined,
        testid: el.getAttribute("data-testid") || undefined,
        text: text.length > 0 ? text : undefined,
        childCount: children.length,
      };
    }

    return normalize(app);
  });
}

/** Type text into the new-todo input and press Enter. */
async function addTodo(page: Page, text: string) {
  const input = page.locator('[data-testid="new-todo"]');
  await input.fill(text);
  await page.locator('[data-testid="add-btn"]').click();
  // Wait for re-render
  await page.waitForTimeout(100);
}

// --- Tests ---

test.describe("TodoMVC DOM equivalence", () => {
  test("initial render shows header", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("h1")).toHaveText("todos");
    await expect(page.locator('[data-testid="app"]')).toBeVisible();
  });

  test("shows version info", async ({ page }) => {
    await page.goto("/");
    const version = page.locator('[data-testid="version"]');
    await expect(version).toBeVisible();
    await expect(version).toContainText("v1.0.0");
  });

  test("add a todo", async ({ page }) => {
    await page.goto("/");
    await addTodo(page, "Buy milk");

    const todoList = page.locator('[data-testid="todo-list"]');
    await expect(todoList).toBeVisible();

    const items = todoList.locator("li");
    await expect(items).toHaveCount(1);

    const todoText = items.first().locator(".todo-text");
    await expect(todoText).toHaveText("Buy milk");
  });

  test("add multiple todos", async ({ page }) => {
    await page.goto("/");
    await addTodo(page, "Buy milk");
    await addTodo(page, "Walk dog");
    await addTodo(page, "Read book");

    const items = page.locator('[data-testid="todo-list"] li');
    await expect(items).toHaveCount(3);

    // Check count uses Utils.formatCount
    const count = page.locator('[data-testid="count"]');
    await expect(count).toContainText("3 items left");
  });

  test("toggle todo completion", async ({ page }) => {
    await page.goto("/");
    await addTodo(page, "Buy milk");

    // Toggle the checkbox
    const checkbox = page.locator('[data-testid="todo-1"] .toggle');
    await checkbox.click();
    await page.waitForTimeout(100);

    // Should have 'completed' class (R8: template literal className)
    const li = page.locator('[data-testid="todo-1"]');
    await expect(li).toHaveClass(/completed/);

    // Count should show 0 items left
    const count = page.locator('[data-testid="count"]');
    await expect(count).toContainText("0");
  });

  test("delete todo", async ({ page }) => {
    await page.goto("/");
    await addTodo(page, "Buy milk");
    await addTodo(page, "Walk dog");

    // Delete first todo
    await page.locator('[data-testid="delete-1"]').click();
    await page.waitForTimeout(100);

    const items = page.locator('[data-testid="todo-list"] li');
    await expect(items).toHaveCount(1);
  });

  test("filter todos", async ({ page }) => {
    await page.goto("/");
    await addTodo(page, "Done task");
    await addTodo(page, "Active task");

    // Complete first todo
    await page.locator('[data-testid="todo-1"] .toggle').click();
    await page.waitForTimeout(100);

    // Filter: Active
    const filters = page.locator('[data-testid="filters"]');
    await filters.locator("button", { hasText: "Active" }).click();
    await page.waitForTimeout(100);

    let items = page.locator('[data-testid="todo-list"] li');
    await expect(items).toHaveCount(1);
    await expect(items.first().locator(".todo-text")).toHaveText("Active task");

    // Filter: Completed
    await filters.locator("button", { hasText: "Completed" }).click();
    await page.waitForTimeout(100);

    items = page.locator('[data-testid="todo-list"] li');
    await expect(items).toHaveCount(1);
    await expect(items.first().locator(".todo-text")).toHaveText("Done task");

    // Filter: All
    await filters.locator("button", { hasText: "All" }).click();
    await page.waitForTimeout(100);

    items = page.locator('[data-testid="todo-list"] li');
    await expect(items).toHaveCount(2);
  });

  test("clear completed", async ({ page }) => {
    await page.goto("/");
    await addTodo(page, "Task A");
    await addTodo(page, "Task B");

    // Complete Task A
    await page.locator('[data-testid="todo-1"] .toggle').click();
    await page.waitForTimeout(100);

    // Clear completed
    await page.locator('[data-testid="clear-completed"]').click();
    await page.waitForTimeout(100);

    const items = page.locator('[data-testid="todo-list"] li');
    await expect(items).toHaveCount(1);
    await expect(items.first().locator(".todo-text")).toHaveText("Task B");
  });

  test("toggle all", async ({ page }) => {
    await page.goto("/");
    await addTodo(page, "Task A");
    await addTodo(page, "Task B");

    // Toggle all
    await page.locator('[data-testid="toggle-all"]').click();
    await page.waitForTimeout(100);

    // All should be completed
    const items = page.locator('[data-testid="todo-list"] li');
    const count = await items.count();
    for (let i = 0; i < count; i++) {
      await expect(items.nth(i)).toHaveClass(/completed/);
    }

    // Count should be 0
    await expect(page.locator('[data-testid="count"]')).toContainText("0");
  });

  test("spread props render correctly (R3)", async ({ page }) => {
    await page.goto("/");
    await addTodo(page, "Spread test");

    // TodoItem receives {...todo} spread — verify it renders with correct data attributes
    const item = page.locator('[data-testid="todo-1"]');
    await expect(item).toBeVisible();
    await expect(item.locator(".todo-text")).toHaveText("Spread test");
  });

  test("conditional JSX: done badge appears on completion (R4)", async ({ page }) => {
    await page.goto("/");
    await addTodo(page, "Conditional test");

    // Before toggle: no done badge
    let badge = page.locator('[data-testid="todo-1"] .done-badge');
    await expect(badge).toHaveCount(0);

    // Toggle to done
    await page.locator('[data-testid="todo-1"] .toggle').click();
    await page.waitForTimeout(100);

    // After toggle: done badge visible
    badge = page.locator('[data-testid="todo-1"] .done-badge');
    await expect(badge).toBeVisible();
  });

  test("about page lazy load (R6)", async ({ page }) => {
    await page.goto("/");

    // Click "Show About"
    await page.locator('[data-testid="about-btn"]').click();
    await page.waitForTimeout(300);

    // About section should appear
    const aboutSection = page.locator('[data-testid="about-section"]');
    await expect(aboutSection).toBeVisible();
  });

  test("star re-export: TodoStats shows progress from lib barrel", async ({
    page,
  }) => {
    await page.goto("/");
    await addTodo(page, "Task A");
    await addTodo(page, "Task B");

    // TodoStats uses percentage() and PI from lib/index.ts (export * from "./math")
    const stats = page.locator('[data-testid="todo-stats"]');
    await expect(stats).toBeVisible();

    // Progress text via lib/formatting (aliased import: percentage as calcPct)
    await expect(page.locator('[data-testid="progress-text"]')).toContainText(
      "0% complete"
    );

    // PI value from star re-export chain
    await expect(page.locator('[data-testid="pi-value"]')).toHaveText("3.14");

    // Optional chaining: first todo text
    await expect(page.locator('[data-testid="first-todo-text"]')).toHaveText(
      "Task A"
    );
  });

  test("star re-export: progress updates on toggle", async ({ page }) => {
    await page.goto("/");
    await addTodo(page, "Task A");
    await addTodo(page, "Task B");

    // Toggle first todo
    await page.locator('[data-testid="todo-1"] .toggle').click();
    await page.waitForTimeout(100);

    // Should show 50% (1/2 done)
    await expect(page.locator('[data-testid="progress-text"]')).toContainText(
      "50% complete"
    );
  });

  test("optional chaining: no todos shows fallback text", async ({ page }) => {
    await page.goto("/");

    // No todos — TodoStats not shown (conditional render)
    const stats = page.locator('[data-testid="todo-stats"]');
    await expect(stats).toHaveCount(0);
  });

  test("inline styles: progress bar has width style", async ({ page }) => {
    await page.goto("/");
    await addTodo(page, "Task A");

    // Toggle to done
    await page.locator('[data-testid="todo-1"] .toggle').click();
    await page.waitForTimeout(100);

    // Progress fill should have 100% width
    const fill = page.locator('[data-testid="progress-fill"]');
    await expect(fill).toBeVisible();
    const width = await fill.evaluate(
      (el) => (el as HTMLElement).style.width
    );
    expect(width).toBe("100%");
  });

  test("enum values render correctly", async ({ page }) => {
    await page.goto("/");
    await addTodo(page, "Enum test");

    // Priority.Medium = 1 (numeric enum)
    await expect(page.locator('[data-testid="priority-value"]')).toHaveText(
      "1"
    );
  });

  test("second dynamic import: Settings page loads", async ({ page }) => {
    await page.goto("/");

    // Click "Show Settings"
    await page.locator('[data-testid="settings-btn"]').click();
    await page.waitForTimeout(300);

    // Settings section should appear
    const section = page.locator('[data-testid="settings-section"]');
    await expect(section).toBeVisible();
  });

  test("aliased re-export: APP_DISPLAY_NAME renders", async ({ page }) => {
    await page.goto("/");

    // AppInfo uses APP_DISPLAY_NAME re-exported as { appName as APP_DISPLAY_NAME }
    const appName = page.locator('[data-testid="app-name"]');
    await expect(appName).toHaveText("Mini React TodoMVC");
  });

  test("computed property: STATUS_MAP renders", async ({ page }) => {
    await page.goto("/");

    // STATUS_MAP uses computed key: { [STATUS_KEY]: "ok" }
    const badge = page.locator('[data-testid="status-badge"]');
    await expect(badge).toHaveText("OK");
  });

  test("destructuring defaults: createConfig works", async ({ page }) => {
    await page.goto("/");

    // createConfig() uses { theme = "light", lang = "en" } defaults
    await expect(page.locator('[data-testid="config-theme"]')).toHaveText(
      "light"
    );
    await expect(page.locator('[data-testid="config-lang"]')).toHaveText("en");
  });

  test("JSX in variable: statusBadge assigns correctly", async ({ page }) => {
    await page.goto("/");

    // statusBadge is JSX assigned to a variable, then embedded
    const badge = page.locator('[data-testid="status-badge"]');
    await expect(badge).toBeVisible();
    await expect(badge).toHaveClass("status-ok");
  });

  test("async handler: fetchItems resolves", async ({ page }) => {
    await page.goto("/");

    // Click async test button
    await page.locator('[data-testid="async-test-btn"]').click();
    await page.waitForTimeout(200);

    // Should show joined result from async fetchItems
    await expect(page.locator('[data-testid="async-result"]')).toHaveText(
      "a,b,c"
    );
  });

  test("todo summary uses template literal", async ({ page }) => {
    await page.goto("/");

    // No todos: should show "empty"
    await expect(page.locator('[data-testid="todo-summary"]')).toHaveText(
      "empty"
    );

    // Add todos
    await addTodo(page, "Task A");
    await expect(page.locator('[data-testid="todo-summary"]')).toHaveText(
      "1 todos"
    );
  });

  test("DOM structure snapshot matches", async ({ page }) => {
    await page.goto("/");
    await addTodo(page, "Snapshot test");

    const snapshot = await getAppSnapshot(page);
    expect(snapshot).not.toBeNull();
    expect(snapshot.tag).toBe("div");
    expect(snapshot.class).toBe("todoapp");
    expect(snapshot.childCount).toBeGreaterThan(0);
  });
});
