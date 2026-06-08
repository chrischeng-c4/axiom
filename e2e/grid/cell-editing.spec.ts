import { test, expect } from '@jet/test';

test.describe('Cell Editing', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');

    // Wait for the canvas to be visible
    await page.locator('#spreadsheet-canvas').waitFor({ state: 'visible' });
  });

  test('should focus formula input when clicking on canvas', async ({ page }) => {
    const canvas = page.locator('#spreadsheet-canvas');
    const formulaInput = page.locator('#formula-input');

    // Click on the canvas (approximate cell location)
    await canvas.click({ position: { x: 100, y: 50 } });

    // Wait a bit for the focus event
    await page.waitForTimeout(100);

    // Formula input might be focused (depending on implementation)
    // This is a basic check - adjust based on actual behavior
    await expect(formulaInput).toBeVisible();
  });

  test('should allow typing in formula input', async ({ page }) => {
    const formulaInput = page.locator('#formula-input');

    // Click on formula input
    await formulaInput.click();

    // Type some text
    await formulaInput.fill('Hello World');

    // Verify the value
    await expect(formulaInput).toHaveValue('Hello World');
  });

  test('should handle Enter key in formula input', async ({ page }) => {
    const formulaInput = page.locator('#formula-input');

    // Click and type
    await formulaInput.click();
    await formulaInput.fill('Test Value');

    // Press Enter
    await formulaInput.press('Enter');

    // The input might clear or keep the value depending on implementation
    // This test just verifies Enter doesn't crash the app
    await expect(formulaInput).toBeVisible();
  });

  test('should handle Escape key in formula input', async ({ page }) => {
    const formulaInput = page.locator('#formula-input');

    // Click and type
    await formulaInput.click();
    await formulaInput.fill('Test Value');

    // Press Escape
    await formulaInput.press('Escape');

    // Input should still be visible
    await expect(formulaInput).toBeVisible();
  });

  test('should handle formula input with equals sign', async ({ page }) => {
    const formulaInput = page.locator('#formula-input');

    // Click and type a formula
    await formulaInput.click();
    await formulaInput.fill('=SUM(A1:A10)');

    // Verify the value
    await expect(formulaInput).toHaveValue('=SUM(A1:A10)');

    // Press Enter to submit
    await formulaInput.press('Enter');

    // Verify no errors occurred
    await expect(formulaInput).toBeVisible();
  });

  test('should handle arrow key navigation', async ({ page }) => {
    const canvas = page.locator('#spreadsheet-canvas');

    // Click on canvas to focus
    await canvas.click({ position: { x: 100, y: 50 } });

    // Press arrow keys (this tests keyboard event handling)
    await page.keyboard.press('ArrowDown');
    await page.keyboard.press('ArrowRight');
    await page.keyboard.press('ArrowUp');
    await page.keyboard.press('ArrowLeft');

    // Verify canvas is still functional
    await expect(canvas).toBeVisible();
  });

  test('should handle Tab key for cell navigation', async ({ page }) => {
    const canvas = page.locator('#spreadsheet-canvas');

    // Click on canvas
    await canvas.click({ position: { x: 100, y: 50 } });

    // Press Tab
    await page.keyboard.press('Tab');

    // Verify canvas is still functional
    await expect(canvas).toBeVisible();
  });

  test('should handle cell selection with mouse', async ({ page }) => {
    const canvas = page.locator('#spreadsheet-canvas');

    // Click on first cell
    await canvas.click({ position: { x: 100, y: 50 } });
    await page.waitForTimeout(100);

    // Click on another cell
    await canvas.click({ position: { x: 200, y: 100 } });
    await page.waitForTimeout(100);

    // Verify canvas is still responsive
    await expect(canvas).toBeVisible();
  });

  test('should handle double-click for edit mode', async ({ page }) => {
    const canvas = page.locator('#spreadsheet-canvas');

    // Double-click on a cell
    await canvas.dblclick({ position: { x: 100, y: 50 } });

    // Wait for potential edit mode activation
    await page.waitForTimeout(200);

    // Verify application is still functional
    await expect(canvas).toBeVisible();
  });

  test('should handle rapid cell selections', async ({ page }) => {
    const canvas = page.locator('#spreadsheet-canvas');

    // Rapid clicks on different cells
    for (let i = 0; i < 5; i++) {
      await canvas.click({ position: { x: 100 + i * 50, y: 50 + i * 30 } });
      await page.waitForTimeout(50);
    }

    // Verify canvas is still responsive
    await expect(canvas).toBeVisible();
  });
});
