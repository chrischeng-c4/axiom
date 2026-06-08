import * as fs from "node:fs";
import * as path from "node:path";
import { Page } from "@jet/test";

export const FIXTURE_DIR = path.resolve(__dirname, "..");

/** Read a fixture source file. */
export function readFixture(relativePath: string): string {
  return fs.readFileSync(path.join(FIXTURE_DIR, relativePath), "utf-8");
}

/** Write a fixture source file. */
export function writeFixture(relativePath: string, content: string): void {
  fs.writeFileSync(path.join(FIXTURE_DIR, relativePath), content, "utf-8");
}

/** Add a todo item to establish app state. */
export async function addTodo(page: Page, text: string) {
  const input = page.locator('[data-testid="new-todo"]');
  await input.fill(text);
  await page.locator('[data-testid="add-btn"]').click();
  await page.waitForTimeout(100);
}
