// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-examples-test-specs.md#tests
// CODEGEN-BEGIN
// @ts-nocheck — documentation-only spec; globals are injected at runtime.
// Page Object pattern example for the jet native test runner.
//
// A Page Object encapsulates a view's selectors and user-facing actions
// behind a typed interface, so tests assert on behaviour rather than CSS
// class names. Combine with `test.extend` to construct the page object once
// per test.
//
// This file is NOT executed by cargo — it is a documentation-only spec that
// demonstrates the recommended structure for migrating Playwright-style
// e2e specs onto the jet runner. See `.aw/tech-design/projects/jet/logic/test-runner.md`
// for the full Expect Matcher Registry and Fixture Lifecycle API.
//
// @spec .aw/changes/enhancement-phase-3-dom-matchers-fixtures-for-native-test-runn/specs/enhancement-phase-3-dom-matchers-fixtures-for-native-test-runn-spec.md#R9

// Jet runtime injects these as globals — declare for type-checking.
declare const describe: (name: string, body: () => void) => void;
type JetTest = {
  (name: string, body: (args?: Record<string, unknown>) => Promise<void> | void): void;
  extend: (fixtures: Record<string, (use: (value: unknown) => Promise<void>) => Promise<void>>) => JetTest;
};
declare const test: JetTest;
declare const expect: (actual: unknown) => {
  toBe: (v: unknown) => void;
  toHaveText: (sel: string, expected: string | RegExp, opts?: { timeout?: number }) => Promise<void>;
  toBeVisible: (sel: string, opts?: { timeout?: number }) => Promise<void>;
  toMatchSnapshot: (name?: string) => Promise<void>;
};

type Page = { __jet_page_id: string; goto: (url: string) => Promise<void> };

class LoginPage {
  constructor(private page: Page) {}

  async goto() {
    await this.page.goto("/login");
  }

  // Selectors are co-located with the actions that use them.
  readonly email = "input[name=email]";
  readonly password = "input[name=password]";
  readonly submit = "button[type=submit]";
}

// A fixture wraps the page object construction so every test that needs it
// gets a fresh instance. Flat fixtures (no cross-fixture deps) are supported
// by `test.extend` in jet's Phase 3 runtime.
const testWithLogin = test.extend({
  loginPage: async (use: (lp: LoginPage) => Promise<void>) => {
    const page = (globalThis as unknown as { page: Page }).page;
    const lp = new LoginPage(page);
    await lp.goto();
    await use(lp);
    // Teardown (if any) goes after `use` — runs before the next test.
  },
});

describe("login flow", () => {
  testWithLogin("shows email and password fields", async ({ loginPage }) => {
    const page = (globalThis as unknown as { page: Page }).page;
    await expect(page).toBeVisible(loginPage.email);
    await expect(page).toBeVisible(loginPage.password);
    await expect(page).toHaveText(loginPage.submit, "Sign in");
  });

  testWithLogin("matches the login screenshot", async ({ loginPage }) => {
    const page = (globalThis as unknown as { page: Page }).page;
    await expect(page).toMatchSnapshot("login-page");
  });
});
// CODEGEN-END
