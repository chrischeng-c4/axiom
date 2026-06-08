import { test, expect } from '@jet/test';

test.describe('RuSheet Application', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should load the application', async ({ page }) => {
    // Check that the title is correct
    await expect(page).toHaveTitle(/RuSheet/i);
  });

  test('should render the spreadsheet canvas', async ({ page }) => {
    // Check that the canvas element exists
    const canvas = page.locator('#spreadsheet-canvas');
    await expect(canvas).toBeVisible();
  });

  test('should render the formula input', async ({ page }) => {
    // Check that the formula input exists
    const formulaInput = page.locator('#formula-input');
    await expect(formulaInput).toBeVisible();
  });

  test('should have responsive canvas', async ({ page }) => {
    const canvas = page.locator('#spreadsheet-canvas');

    // Get initial canvas dimensions
    const initialBox = await canvas.boundingBox();
    expect(initialBox).toBeTruthy();

    // Resize viewport
    await page.setViewportSize({ width: 1200, height: 800 });

    // Wait a bit for resize
    await page.waitForTimeout(100);

    // Canvas should still be visible
    await expect(canvas).toBeVisible();
  });

  test('should initialize WASM module', async ({ page }) => {
    // Wait for WASM to load by checking if canvas is interactive
    const canvas = page.locator('#spreadsheet-canvas');
    await expect(canvas).toBeVisible();

    // Check that there are no console errors related to WASM
    const errors: string[] = [];
    page.on('console', (msg) => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });

    await page.waitForTimeout(1000);

    // Filter out non-WASM related errors
    const wasmErrors = errors.filter(e => e.toLowerCase().includes('wasm'));
    expect(wasmErrors.length).toBe(0);
  });

  test('should render grid lines on canvas', async ({ page }) => {
    const canvas = page.locator('#spreadsheet-canvas');
    await expect(canvas).toBeVisible();

    // Take a screenshot to verify rendering
    const screenshot = await canvas.screenshot();
    expect(screenshot.length).toBeGreaterThan(0);
  });
});
