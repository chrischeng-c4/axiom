---
id: stdlib-testing
type: spec
title: "stdlib: unittest"
version: 1
spec_type: utility
files:
  - runtime/stdlib/unittest_mod.rs
---

# stdlib: unittest

Unit testing framework modeled after Python's `unittest` module. Provides test
case authoring, assertion helpers, discovery, and result reporting.

## Requirements

### R1: TestCase base class with setUp / tearDown

Users subclass `TestCase` and define `test_*` methods.

**Acceptance criteria**
- `setUp()` runs before each test method.
- `tearDown()` runs after each test method, even if the test fails.
- `setUpClass()` / `tearDownClass()` run once per class (class methods).
- A test method whose name starts with `test` is collected automatically.

### R2: Assert methods

Standard assertion helpers on `TestCase`.

**Acceptance criteria**
- `assertEqual(a, b)` — fails if `a != b`, message shows both values.
- `assertNotEqual(a, b)` — fails if `a == b`.
- `assertTrue(x)` — fails if `bool(x)` is `False`.
- `assertFalse(x)` — fails if `bool(x)` is `True`.
- `assertRaises(ExcType)` — context manager that expects the exception.
- `assertIn(a, b)` — fails if `a not in b`.
- `assertIsNone(x)` — fails if `x is not None`.
- `assertIsNotNone(x)` — fails if `x is None`.
- All assertions accept an optional `msg` parameter for custom messages.

### R3: Test discovery and runner

Discover and execute tests.

**Acceptance criteria**
- `unittest.main()` discovers all `TestCase` subclasses in the calling module.
- `TestLoader().loadTestsFromTestCase(cls)` loads tests from a specific class.
- `TestSuite` aggregates multiple tests or sub-suites.
- `TextTestRunner(verbosity=N)` runs a suite and prints results.
- Tests run in alphabetical order within each class.

### R4: Test result reporting

Track and display test outcomes.

**Acceptance criteria**
- `TestResult` records counts for: tests run, failures, errors, skipped.
- Failures show the assertion message and traceback.
- Errors (unexpected exceptions) show the full traceback.
- Summary line format: `Ran N tests in X.XXXs` followed by `OK` or `FAILED`.
- Exit code is 0 on success, 1 on any failure or error.

## Non-goals

- `unittest.mock` (separate spec).
- `@skip` / `@expectedFailure` decorators (deferred).
- Async test support (deferred).

## Dependencies

- OOP model (class inheritance, method resolution).
- Exception hierarchy (for `assertRaises`).
- traceback module (for failure formatting).
