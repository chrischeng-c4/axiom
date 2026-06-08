// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests-fixtures-playwright-compat.md#tests
// CODEGEN-BEGIN
// Fixture spec for jet test --playwright end-to-end test (T9).
// Imports @playwright/test to verify import-based routing.
// REQ: R9

import { test, expect } from '@playwright/test';

test('works', async () => {
  // Minimal passing test — no browser needed.
  expect(1 + 1).toBe(2);
});
// CODEGEN-END
