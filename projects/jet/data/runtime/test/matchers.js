// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-data-runtime-test.md#logic
// CODEGEN-BEGIN
// Polling expect matchers for the jet native test runner.
//
// Each matcher polls its probe function at 100ms intervals until the predicate
// holds or the timeout (default 5000ms) elapses, then throws AssertionError.
//
// Shared helper: pollUntil(probe, predicate, timeout, buildError)
//
// Imported by index.js and attached to the expect() return object.
//
// @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R20
// @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R21
// @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R22
// @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R23
// @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R24
// @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R25
// @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R26
// @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R27

class AssertionError extends Error {
  constructor(message, diff) {
    super(message);
    this.name = "AssertionError";
    this.diff = diff;
  }
}

function sleep(ms) {
  return new Promise((r) => setTimeout(r, ms));
}

/**
 * Shared polling helper for all matchers.
 *
 * @param {() => Promise<any>} probe - Async function that reads the current value.
 * @param {(value: any) => boolean} predicate - Returns true when condition is met.
 * @param {number} timeout - Total wait time in milliseconds (default 5000).
 * @param {(lastValue: any, lastError: Error|null) => AssertionError} buildError - Called on timeout.
 * @returns {Promise<void>} Resolves when predicate holds.
 */
async function pollUntil(probe, predicate, timeout, buildError) {
  const start = Date.now();
  let lastValue = undefined;
  let lastError = null;

  while (true) {
    try {
      lastValue = await probe();
      if (predicate(lastValue)) return;
      lastError = null;
    } catch (err) {
      lastError = err;
    }

    if (Date.now() - start >= timeout) {
      throw buildError(lastValue, lastError);
    }

    await sleep(100);
  }
}

/**
 * Check whether a value matches a string or regex pattern.
 */
function matchesPattern(actual, expected) {
  if (expected instanceof RegExp) {
    return typeof actual === "string" && expected.test(actual);
  }
  return actual === expected;
}

function containsPattern(actual, expected) {
  if (expected instanceof RegExp) {
    if (typeof actual !== "string") return false;
    expected.lastIndex = 0;
    return expected.test(actual);
  }
  return typeof actual === "string" && actual.includes(String(expected));
}

function describeLocator(locator) {
  if (locator && typeof locator.toString === "function") {
    return locator.toString();
  }
  return "locator";
}

// ── Page matchers ────────────────────────────────────────────────────────────

/**
 * expect(page).toHaveTitle(string|regex, opts?) — polls page.title().
 *
 * @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R20
 */
export async function toHaveTitle(page, expected, opts) {
  const timeout = (opts && opts.timeout) != null ? opts.timeout : 5000;
  return pollUntil(
    () => page.title(),
    (title) => matchesPattern(title, expected),
    timeout,
    (lastValue, lastError) => {
      const msg = lastError
        ? `toHaveTitle: ${lastError.message ?? String(lastError)}`
        : `Expected title to match ${displayPattern(expected)}, got ${JSON.stringify(lastValue)}`;
      return new AssertionError(msg, `- expected: ${displayPattern(expected)}\n+ actual:   ${JSON.stringify(lastValue)}`);
    },
  );
}

/**
 * expect(page).toHaveURL(string|regex, opts?) — polls page.url().
 *
 * @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R21
 */
export async function toHaveURL(page, expected, opts) {
  const timeout = (opts && opts.timeout) != null ? opts.timeout : 5000;
  return pollUntil(
    () => page.url(),
    (url) => matchesPattern(url, expected),
    timeout,
    (lastValue, lastError) => {
      const msg = lastError
        ? `toHaveURL: ${lastError.message ?? String(lastError)}`
        : `Expected URL to match ${displayPattern(expected)}, got ${JSON.stringify(lastValue)}`;
      return new AssertionError(msg, `- expected: ${displayPattern(expected)}\n+ actual:   ${JSON.stringify(lastValue)}`);
    },
  );
}

// ── Locator matchers ─────────────────────────────────────────────────────────

/**
 * expect(locator).toBeVisible(opts?) — polls locator.isVisible().
 *
 * @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R22
 */
export async function toBeVisibleLocator(locator, opts) {
  const timeout = (opts && opts.timeout) != null ? opts.timeout : 5000;
  return pollUntil(
    () => locator.isVisible(),
    (v) => v === true,
    timeout,
    (_lastValue, lastError) => {
      const msg = lastError
        ? `toBeVisible: ${lastError.message ?? String(lastError)}`
        : `Expected ${describeLocator(locator)} to be visible within ${timeout}ms`;
      return new AssertionError(msg);
    },
  );
}

/**
 * expect(locator).toBeHidden(opts?) — polls locator.isVisible() (negated).
 *
 * @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R22
 */
export async function toBeHidden(locator, opts) {
  const timeout = (opts && opts.timeout) != null ? opts.timeout : 5000;
  return pollUntil(
    () => locator.isVisible(),
    (v) => v === false,
    timeout,
    (_lastValue, lastError) => {
      const msg = lastError
        ? `toBeHidden: ${lastError.message ?? String(lastError)}`
        : `Expected element to be hidden within ${timeout}ms`;
      return new AssertionError(msg);
    },
  );
}

/**
 * expect(locator).toHaveText(string|regex, opts?) — polls locator.innerText().
 *
 * @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R23
 */
export async function toHaveTextLocator(locator, expected, opts) {
  const timeout = (opts && opts.timeout) != null ? opts.timeout : 5000;
  return pollUntil(
    () => locator.innerText(),
    (text) => matchesPattern(text, expected),
    timeout,
    (lastValue, lastError) => {
      const msg = lastError
        ? `toHaveText: ${lastError.message ?? String(lastError)}`
        : `Expected element text to match ${displayPattern(expected)}, got ${JSON.stringify(lastValue)}`;
      return new AssertionError(msg, `- expected: ${displayPattern(expected)}\n+ actual:   ${JSON.stringify(lastValue)}`);
    },
  );
}

/**
 * expect(locator).toContainText(string|regex, opts?) — polls locator.innerText().
 */
export async function toContainTextLocator(locator, expected, opts) {
  const timeout = (opts && opts.timeout) != null ? opts.timeout : 5000;
  return pollUntil(
    () => locator.innerText(),
    (text) => containsPattern(text, expected),
    timeout,
    (lastValue, lastError) => {
      const msg = lastError
        ? `toContainText: ${lastError.message ?? String(lastError)}`
        : `Expected element text to contain ${displayPattern(expected)}, got ${JSON.stringify(lastValue)}`;
      return new AssertionError(msg, `- expected contains: ${displayPattern(expected)}\n+ actual:            ${JSON.stringify(lastValue)}`);
    },
  );
}

/**
 * expect(locator).toHaveValue(string, opts?) — polls locator.inputValue().
 *
 * @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R24
 */
export async function toHaveValue(locator, expected, opts) {
  const timeout = (opts && opts.timeout) != null ? opts.timeout : 5000;
  return pollUntil(
    () => locator.inputValue(),
    (value) => value === expected,
    timeout,
    (lastValue, lastError) => {
      const msg = lastError
        ? `toHaveValue: ${lastError.message ?? String(lastError)}`
        : `Expected input value to be ${JSON.stringify(expected)}, got ${JSON.stringify(lastValue)}`;
      return new AssertionError(msg, `- expected: ${JSON.stringify(expected)}\n+ actual:   ${JSON.stringify(lastValue)}`);
    },
  );
}

/**
 * expect(locator).toHaveCount(n, opts?) — polls locator.count().
 *
 * @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R25
 */
export async function toHaveCount(locator, expected, opts) {
  const timeout = (opts && opts.timeout) != null ? opts.timeout : 5000;
  return pollUntil(
    () => locator.count(),
    (count) => count === expected,
    timeout,
    (lastValue, lastError) => {
      const msg = lastError
        ? `toHaveCount: ${lastError.message ?? String(lastError)}`
        : `Expected element count to be ${expected}, got ${lastValue}`;
      return new AssertionError(msg, `- expected: ${expected}\n+ actual:   ${lastValue}`);
    },
  );
}

/**
 * expect(locator).toHaveClass(string|regex, opts?) — polls element.className.
 *
 * @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R26
 */
export async function toHaveClass(locator, expected, opts) {
  const timeout = (opts && opts.timeout) != null ? opts.timeout : 5000;
  const probe = async () => {
    // Use getAttribute("class") via the existing getAttribute method.
    const cls = await locator.getAttribute("class");
    return cls || "";
  };
  const predicate = (className) => {
    if (expected instanceof RegExp) return expected.test(className);
    // String: check if className contains the expected class token.
    return className === expected || className.split(/\s+/).includes(expected);
  };
  return pollUntil(
    probe,
    predicate,
    timeout,
    (lastValue, lastError) => {
      const msg = lastError
        ? `toHaveClass: ${lastError.message ?? String(lastError)}`
        : `Expected element class to match ${displayPattern(expected)}, got ${JSON.stringify(lastValue)}`;
      return new AssertionError(msg, `- expected: ${displayPattern(expected)}\n+ actual:   ${JSON.stringify(lastValue)}`);
    },
  );
}

/**
 * expect(locator).toHaveAttribute(name, value, opts?) — polls getAttribute.
 *
 * @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R27
 */
export async function toHaveAttribute(locator, name, expected, opts) {
  const timeout = (opts && opts.timeout) != null ? opts.timeout : 5000;
  return pollUntil(
    () => locator.getAttribute(name),
    (value) => matchesPattern(value, expected),
    timeout,
    (lastValue, lastError) => {
      const msg = lastError
        ? `toHaveAttribute(${JSON.stringify(name)}): ${lastError.message ?? String(lastError)}`
        : `Expected attribute ${JSON.stringify(name)} to match ${displayPattern(expected)}, got ${JSON.stringify(lastValue)}`;
      return new AssertionError(msg, `- expected: ${displayPattern(expected)}\n+ actual:   ${JSON.stringify(lastValue)}`);
    },
  );
}

// ── Phase 5 matchers: element-state + CSS + accessibility ────────────────────

/**
 * expect(locator).toBeChecked(opts?) — polls locator.isChecked().
 * @spec .aw/tech-design/projects/jet/logic/matchers-state-value-a11y.md#M1
 */
export async function toBeChecked(locator, opts) {
  const timeout = (opts && opts.timeout) != null ? opts.timeout : 5000;
  return pollUntil(
    () => locator.isChecked(),
    (v) => v === true,
    timeout,
    (_l, lastError) => new AssertionError(
      lastError ? `toBeChecked: ${lastError.message ?? String(lastError)}`
                : `Expected element to be checked within ${timeout}ms`,
    ),
  );
}

/**
 * expect(locator).toBeDisabled(opts?) — polls locator.isDisabled().
 * @spec matchers-state-value-a11y#M2
 */
export async function toBeDisabled(locator, opts) {
  const timeout = (opts && opts.timeout) != null ? opts.timeout : 5000;
  return pollUntil(
    () => locator.isDisabled(),
    (v) => v === true,
    timeout,
    (_l, lastError) => new AssertionError(
      lastError ? `toBeDisabled: ${lastError.message ?? String(lastError)}`
                : `Expected element to be disabled within ${timeout}ms`,
    ),
  );
}

/**
 * expect(locator).toBeEnabled(opts?) — polls locator.isEnabled().
 * @spec matchers-state-value-a11y#M3
 */
export async function toBeEnabled(locator, opts) {
  const timeout = (opts && opts.timeout) != null ? opts.timeout : 5000;
  return pollUntil(
    () => locator.isEnabled(),
    (v) => v === true,
    timeout,
    (_l, lastError) => new AssertionError(
      lastError ? `toBeEnabled: ${lastError.message ?? String(lastError)}`
                : `Expected element to be enabled within ${timeout}ms`,
    ),
  );
}

/**
 * expect(locator).toBeFocused(opts?) — polls locator.isFocused().
 * @spec matchers-state-value-a11y#M4
 */
export async function toBeFocused(locator, opts) {
  const timeout = (opts && opts.timeout) != null ? opts.timeout : 5000;
  return pollUntil(
    () => locator.isFocused(),
    (v) => v === true,
    timeout,
    (_l, lastError) => new AssertionError(
      lastError ? `toBeFocused: ${lastError.message ?? String(lastError)}`
                : `Expected element to be focused within ${timeout}ms`,
    ),
  );
}

/**
 * expect(locator).toHaveCSS(name, value|regex, opts?) — polls computed style.
 * @spec matchers-state-value-a11y#M5
 */
export async function toHaveCSS(locator, name, expected, opts) {
  const timeout = (opts && opts.timeout) != null ? opts.timeout : 5000;
  return pollUntil(
    () => locator.computedStyle(name),
    (v) => matchesPattern(v, expected),
    timeout,
    (lastValue, lastError) => new AssertionError(
      lastError ? `toHaveCSS(${JSON.stringify(name)}): ${lastError.message ?? String(lastError)}`
                : `Expected CSS ${JSON.stringify(name)} to match ${displayPattern(expected)}, got ${JSON.stringify(lastValue)}`,
      `- expected: ${displayPattern(expected)}\n+ actual:   ${JSON.stringify(lastValue)}`,
    ),
  );
}

/**
 * expect(locator).toHaveAccessibleName(string|regex, opts?).
 * @spec matchers-state-value-a11y#M6
 */
export async function toHaveAccessibleName(locator, expected, opts) {
  const timeout = (opts && opts.timeout) != null ? opts.timeout : 5000;
  return pollUntil(
    () => locator.accessibleName(),
    (v) => matchesPattern(v, expected),
    timeout,
    (lastValue, lastError) => new AssertionError(
      lastError ? `toHaveAccessibleName: ${lastError.message ?? String(lastError)}`
                : `Expected accessible name to match ${displayPattern(expected)}, got ${JSON.stringify(lastValue)}`,
      `- expected: ${displayPattern(expected)}\n+ actual:   ${JSON.stringify(lastValue)}`,
    ),
  );
}

/**
 * expect(locator).toHaveRole(string, opts?) — ARIA role, explicit or implicit.
 * @spec matchers-state-value-a11y#M7
 */
export async function toHaveRole(locator, expected, opts) {
  const timeout = (opts && opts.timeout) != null ? opts.timeout : 5000;
  return pollUntil(
    () => locator.role(),
    (v) => v === expected,
    timeout,
    (lastValue, lastError) => new AssertionError(
      lastError ? `toHaveRole: ${lastError.message ?? String(lastError)}`
                : `Expected role ${JSON.stringify(expected)}, got ${JSON.stringify(lastValue)}`,
      `- expected: ${JSON.stringify(expected)}\n+ actual:   ${JSON.stringify(lastValue)}`,
    ),
  );
}

// ── Pure-value matcher: toMatchObject ────────────────────────────────────────

/**
 * Recursive partial-object match. Each key in `expected` must exist in
 * `actual` with a matching value. Nested objects are matched partially;
 * arrays must have the same length and each element matched pairwise.
 * RegExp values match if actual is a string the regex matches.
 * @spec matchers-state-value-a11y#M8
 */
export function matchObject(actual, expected) {
  if (expected instanceof RegExp) {
    return typeof actual === "string" && expected.test(actual);
  }
  if (expected === null || typeof expected !== "object") {
    return actual === expected;
  }
  if (Array.isArray(expected)) {
    if (!Array.isArray(actual) || actual.length !== expected.length) return false;
    for (let i = 0; i < expected.length; i++) {
      if (!matchObject(actual[i], expected[i])) return false;
    }
    return true;
  }
  if (actual === null || typeof actual !== "object") return false;
  for (const key of Object.keys(expected)) {
    if (!matchObject(actual[key], expected[key])) return false;
  }
  return true;
}

// ── Helpers ──────────────────────────────────────────────────────────────────

function displayPattern(pattern) {
  if (pattern instanceof RegExp) return pattern.toString();
  try {
    return JSON.stringify(pattern);
  } catch {
    return String(pattern);
  }
}

export { AssertionError };
// CODEGEN-END
