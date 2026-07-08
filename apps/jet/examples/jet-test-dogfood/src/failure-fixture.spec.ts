// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-examples-jet-test-dogfood-src.md#tests
// CODEGEN-BEGIN
// Intentional-failure fixture. The runner must produce structured
// agent-readable result data for this case: `error.message` describing
// what was expected vs actual, plus a stack trace tying the failure to
// this file. The dogfood integration test reads the JSON reporter output
// and asserts the failure shape.

import { describe, test, expect } from "@jet/test";

describe("failure fixture (intentional)", () => {
  test("toBe diff reports actual and expected", () => {
    // Deliberately fails so the JSON reporter emits a `Failed` outcome
    // with `error.message` containing both values.
    expect(2 + 2).toBe(5);
  });
});
// CODEGEN-END
