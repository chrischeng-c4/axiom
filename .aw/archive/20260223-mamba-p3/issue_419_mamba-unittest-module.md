---
number: 419
title: "mamba: unittest module"
state: open
labels: [enhancement, crate:mamba, P3]
dependencies: [407]
---

# #419 — mamba: unittest module

## Description

Implement `unittest` module for test framework support.

## Requirements

- R1: `unittest.TestCase` base class
- R2: Assert methods: `assertEqual`, `assertNotEqual`, `assertTrue`, `assertFalse`, `assertRaises`, `assertIn`, `assertIsNone`, `assertIsInstance`
- R3: `setUp()` / `tearDown()` hooks
- R4: Test discovery and runner: `unittest.main()`
- R5: `@unittest.skip`, `@unittest.skipIf` decorators

## Dependencies

Depends on #407 (metaclasses/abc) for TestCase infrastructure.

## Priority

P3 — important for self-hosted testing but not needed for basic programs.
