// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-examples-jet-test-dogfood-src.md#tests
// CODEGEN-BEGIN
// Unit-style dogfood specs. Pure logic only — no DOM, no fetch, no fixtures.
// All matchers used here are the post-#2605 @jet/test surface.

import { describe, test, expect } from "@jet/test";

function add(a: number, b: number): number {
  return a + b;
}

function slugify(input: string): string {
  return input.trim().toLowerCase().replace(/\s+/g, "-");
}

describe("math", () => {
  test("add returns the sum", () => {
    expect(add(1, 2)).toBe(3);
    expect(add(-1, 1)).toBe(0);
  });

  test("add never produces NaN for finite inputs", () => {
    expect(add(2, 3)).toBeGreaterThan(0);
    expect(add(2, 3)).toBeLessThan(10);
    expect(add(0.1, 0.2)).toBeCloseTo(0.3, 5);
  });
});

describe("slugify", () => {
  test("collapses whitespace into single dashes", () => {
    expect(slugify("Hello   World")).toBe("hello-world");
  });

  test("preserves length characteristics", () => {
    expect(slugify("ab c")).toHaveLength(4);
    expect(slugify("nope")).not.toContain(" ");
  });
});
// CODEGEN-END
