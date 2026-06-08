---
title: cclab-qc Test Framework Overview
status: implemented
component: cclab-qc
type: index
main_spec_ref: "cclab-qc/README.md"
fill_sections: [overview, doc]
---

# cclab-qc Test Framework

## Overview
<!-- type: overview lang: markdown -->

`cclab-qc` is the Rust quality and test framework crate. It provides discovery,
runner metadata, reporting, assertions, fixtures, plugins, benchmarks,
security-test helpers, performance profiling, baselines, and agent-evaluation
utilities. Its optional CLI still serves `cclab probe` style workflows, but the
current execution backend is `python -m pytest` rather than an embedded native-extension
runner.

**Key Features**:
- **Rust Core**: Library APIs live in `crates/cclab-qc/src/`
- **Optional CLI**: Clap command surface lives in `crates/cclab-qc/src/cli/`
- **Fast Discovery**: Rust `jwalk` traversal finds test and benchmark files
- **Pytest Execution**: CLI maps flags to `python -m pytest`
- **Bindings**: Mamba runtime integrations live in sibling crates
- **Code Generation**: Scaffold test/benchmark skeletons via `generate` subcommand

## User Requirements
<!-- type: doc lang: markdown -->

- **CLI**: Rust CliModule (via linkme auto-registration)
- **Discovery**: Rust-powered file traversal and classification
- **Execution**: Rust CLI orchestrates pytest subprocess execution
- **Benchmark**: Keep current BenchmarkGroup pattern
- **Standalone Core**: Keep reusable framework logic outside binding crates

## Architecture
<!-- type: doc lang: markdown -->

```
crates/cclab-qc/src/lib.rs          (Core Rust API and re-exports)
  |-- crates/cclab-qc/src/cli/       (Optional clap CLI)
  |-- crates/cclab-qc/src/discovery.rs
  |-- crates/cclab-qc/src/runner.rs
  |-- crates/cclab-qc/src/reporter.rs
  |-- crates/cclab-qc/src/assertions.rs
  |-- crates/cclab-qc/src/fixtures.rs
  |-- crates/cclab-qc/src/plugin.rs
  |-- crates/cclab-qc/src/agent_eval/

crates/cclab-qc-mamba/              (Mamba native module bindings)
```

**Key Principles**:
1. **Core first**: Framework contracts live in `cclab-qc`.
2. **CLI optional**: The CLI feature depends on clap and keeps subprocess logic in `cli/runner.rs`.
3. **Bindings separate**: Mamba adapters live in sibling crates.
4. **Structured results**: Reporters and baselines consume structured Rust types.

## Documentation Structure
<!-- type: doc lang: markdown -->

This architecture documentation is split into focused files for easier navigation:

### 1. [Architecture Overview](./logic/architecture/overview.md)
High-level system diagrams showing:
- Overall architecture flow with Rust CLI as entry point
- Core crate and binding-crate responsibilities
- Optional CLI ownership
- Layer contracts

### 2. [Framework Lifecycle State Machines](./logic/state-machines/framework-lifecycle.md)
State machine definitions for:
- Discovery traversal and file classification
- Pytest subprocess execution lifecycle
- Collect and migrate CLI workflows
- Current Rust state data structures

### 3. [Data Flows](./logic/flows/data-flows.md)
Sequence diagrams showing:
- Test discovery and execution flow
- Benchmark discovery and execution flow
- Filtering flow
- Error handling
- Performance optimization points

### 4. [Components](./logic/components/overview.md)
Component responsibilities and integration:
- Optional CLI layer
- Core library modules
- Mamba binding crates
- Security, benchmark, baseline, and agent-evaluation components

### 5. [Implementation Details](./logic/implementation/details.md)
Implementation details:
- File structure and organization
- Execution flow diagrams
- Key design patterns
- Performance characteristics
- Future extensions

### 6. [Plugin System](./interfaces/plugin/system.md)
Hook-based plugin interfaces:
- Standard lifecycle hooks
- Registration and priority ordering
- Built-in logging, timeout, and filter plugins

### 7. [Expect Assertion API](./interfaces/expect/api-reference.md)
Fluent assertion interfaces:
- `expect(value)` entry point
- Structured assertion errors
- Equality, option, string, vector, and JSON matchers

### 8. [Fixture DI Integration](./interfaces/fixture/di-integration.md)
Fixture metadata and dependency resolver:
- Fixture scopes and teardown metadata
- Dependency-first ordering
- Cycle detection and autouse lookup

### 9. [Legacy Backend Metadata](./logic/legacy/backend-metadata.md)
Historical result-backend metadata design:
- Not implemented in `cclab-qc`
- Kept for provenance and future migration decisions

## Quick Start
<!-- type: doc lang: markdown -->

### Commands

```bash
cclab probe run python/tests/           # Run all tests
cclab probe run --bench python/benchmarks/orbit/  # Run benchmarks
cclab probe run --coverage              # Run with coverage
cclab probe collect python/tests/       # Collect tests without running
cclab probe migrate python/tests/ --dry-run  # Preview migration
cclab probe generate test user-crud     # Generate test skeleton
cclab probe generate bench event-loop   # Generate bench skeleton
```

### Options (for `cclab probe run`)

```bash
--bench             # Run benchmarks instead of tests
--security          # Run security tests
--coverage          # Collect code coverage
--html              # Output HTML report (use with --coverage)
-o, --output FILE   # Output file for coverage report
--cov-fail-under N  # Fail if coverage below threshold (0-100)
--cov-json          # Output coverage as JSON (for CI tools)
--ci                # CI mode: minimal output, exit codes for automation
-k, --pattern PAT   # Filter tests by pattern (case-insensitive)
-v, --verbose       # Verbose output
--fail-fast         # Stop on first failure
```

## Success Criteria
<!-- type: doc lang: markdown -->

- `cclab probe run` command available after build
- Auto-discovers test_*.py and bench_*.py files
- Filters by pattern
- Runs tests and benchmarks through pytest-compatible subprocess execution
- Generates reports (console/JSON/markdown/HTML)
- <200ms discovery for 100 files
- All existing tests still pass
- Coverage collection with threshold enforcement
- Template generation for test/benchmark files

## References
<!-- type: doc lang: markdown -->

- **Rust Crate**: `crates/cclab-qc/`
- **CLI Module**: `crates/cclab-qc/src/cli/` (mod.rs, runner.rs, generate.rs)
- **Discovery**: `crates/cclab-qc/src/discovery.rs`
- **Reporter**: `crates/cclab-qc/src/reporter.rs`
- **Mamba Bindings**: `crates/cclab-qc-mamba/`
