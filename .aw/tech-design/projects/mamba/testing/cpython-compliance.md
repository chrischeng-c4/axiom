---
id: stdlib-conformance-spec
main_spec_ref: crates/mamba/testing/cpython-compliance.md
merge_strategy: extend
---

# Mamba Merge Tests Spec

## Overview
<!-- type: overview lang: markdown -->

This specification outlines the process for merging the standalone `cclab-mamba-tests` crate into the `cclab-mamba` crate as an internal integration test suite. This consolidation eliminates duplicate test infrastructure, simplifies the workspace structure, and centralizes CPython 3.12 compatibility tracking within the core Mamba parser codebase. The `cpython_compat` test harness and its associated xfail manifest (`known_failures.toml`) will be migrated and updated to reflect their new locations.
## Requirements
<!-- type: overview lang: markdown -->

### R1 - Code Migration
Move `crates/cclab-mamba-tests/tests/cpython_compat.rs` to `crates/mamba/tests/cpython_compat.rs`. Update module references and crate imports as needed to align with being an internal integration test.

### R2 - Fixture Migration
Relocate all Python fixture files from `crates/cclab-mamba-tests/tests/fixtures/cpython/` to `crates/mamba/tests/fixtures/cpython/`. Maintain existing directory structures for fixture discovery.

### R3 - XFail Manifest Migration and Renaming
Move `crates/cclab-mamba-tests/known_failures.toml` to `crates/mamba/cpython_known_failures.toml`. Rename the file to explicitly associate it with the CPython compliance suite and prevent collisions with other test manifests.

### R4 - Crate Deletion
Remove the `cclab-mamba-tests` directory and its reference from the `members` list in the workspace `Cargo.toml`.

### R5 - Integration Test Configuration
Update `crates/mamba/Cargo.toml` to register `cpython_compat` as an integration test target with `harness = false`, ensuring it continues to use its custom `datatest-stable` harness.

### R6 - Manifest Path Integration
Update the `load_xfail_map()` function in `cpython_compat.rs` to correctly resolve the new path and filename (`cpython_known_failures.toml`) of the xfail manifest relative to the `cclab-mamba` crate root.

### R7 - Dependency Management
Ensure `toml` and `datatest-stable` are included in `cclab-mamba`'s `dev-dependencies` to support the migrated test harness.
## Scenarios
<!-- type: overview lang: markdown -->

### Scenario: CPython Compat Test Suite Passes
- **GIVEN** `cclab-mamba` crate with the migrated `cpython_compat` integration test and its associated fixtures
- **WHEN** `cargo test -p mamba --test cpython_compat` is run
- **THEN** all non-xfailed CPython compatibility fixtures parse successfully, and CI passes

### Scenario: XFail Manifest Cleanup Warning (XPASS)
- **GIVEN** a fixture that previously failed is now successfully parsed
- **WHEN** `cargo test -p mamba --test cpython_compat` is run
- **THEN** a warning message `[xpass]` is emitted for the fixture, and CI still passes

### Scenario: Unexpected Failure Detected
- **GIVEN** a previously passing fixture that now fails to parse (e.g., due to a regression)
- **WHEN** `cargo test -p mamba --test cpython_compat` is run
- **THEN** the test fails with a clear message showing the parse error, and CI fails

### Scenario: XFail Manifest correctly skipped
- **GIVEN** a fixture that is listed in `cpython_known_failures.toml`
- **WHEN** `cargo test -p mamba --test cpython_compat` is run
- **THEN** the fixture's failure is recorded with its reason, and the test still passes as it is an expected failure.
## Diagrams
<!-- type: overview lang: markdown -->

## API Spec
<!-- type: overview lang: markdown -->

## Test Plan
<!-- type: overview lang: markdown -->

```mermaid
requirementDiagram
  requirement R1 {
    id: R1
    text: "Move cpython_compat.rs to cclab-mamba/tests"
    risk: low
    verification: Test
  }

  requirement R2 {
    id: R2
    text: "Relocate fixtures to cclab-mamba/tests/fixtures/cpython"
    risk: low
    verification: Test
  }

  requirement R3 {
    id: R3
    text: "Move and rename known_failures.toml to cpython_known_failures.toml"
    risk: low
    verification: Test
  }

  requirement R4 {
    id: R4
    text: "Delete the cclab-mamba-tests crate"
    risk: medium
    verification: Demonstration
  }

  requirement R5 {
    id: R5
    text: "Register cpython_compat as an integration test"
    risk: medium
    verification: Test
  }

  requirement R6 {
    id: R6
    text: "Correct manifest path integration in test code"
    risk: medium
    verification: Test
  }

  element TestCpythonCompatSuite {
    type: "test"
    test_type: "integration"
    given: "cclab-mamba with migrated tests and fixtures"
    when: "cargo test -p mamba --test cpython_compat"
    then: "Tests are run successfully using cpython_known_failures.toml"
  }

  element TestWorkspaceCleanup {
    type: "test"
    test_type: "e2e"
    given: "Modified workspace Cargo.toml and deleted crate directory"
    when: "cargo test --all"
    then: "cclab-mamba-tests is no longer included and tests pass"
  }

  R1 - verifies -> TestCpythonCompatSuite
  R2 - verifies -> TestCpythonCompatSuite
  R3 - verifies -> TestCpythonCompatSuite
  R5 - verifies -> TestCpythonCompatSuite
  R6 - verifies -> TestCpythonCompatSuite
  R4 - verifies -> TestWorkspaceCleanup
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: crates/cclab-mamba-tests/tests/cpython_compat.rs
    action: delete
  - path: crates/cclab-mamba-tests/tests/fixtures/cpython/
    action: delete
  - path: crates/cclab-mamba-tests/known_failures.toml
    action: delete
  - path: crates/cclab-mamba-tests/Cargo.toml
    action: delete
  - path: crates/cclab-mamba-tests/src/lib.rs
    action: delete
  - path: crates/cclab-mamba-tests/
    action: delete
  - path: Cargo.toml
    action: modify
    description: Remove "crates/cclab-mamba-tests" from members.
  - path: crates/mamba/Cargo.toml
    action: modify
    description: Add cpython_compat integration test target and required dev-dependencies.
  - path: crates/mamba/tests/cpython_compat.rs
    action: create
    description: Copy harness, update load_xfail_map path to cpython_known_failures.toml, and update fixture path.
  - path: crates/mamba/tests/fixtures/cpython/
    action: create
    description: Move CPython fixtures from the standalone tests crate.
  - path: crates/mamba/cpython_known_failures.toml
    action: create
    description: Rename known_failures.toml into the mamba crate.
```
