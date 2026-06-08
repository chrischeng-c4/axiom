---
id: improve-probe-maturity
type: exploration
created_at: 2026-01-28T07:22:20.220773+00:00
needs_clarification: false
---

# Codebase Exploration

### Codebase Analysis: cclab-probe Maturity Upgrade

The current implementation of `cclab-probe` provides a solid foundation for test discovery, execution, and reporting. However, it lacks several key features to reach parity with pytest:

1.  **Plugin System:** While `HookType` and `HookRegistry` exist, they are limited to basic lifecycle hooks and are not well-integrated into the main execution flow. A more robust, extensible plugin system (inspired by `pluggy`) is needed.
2.  **Fixture DI Integration:** The `FixtureRegistry` is implemented but disconnected from the `TestRunner`. The runner currently only supports simple `setup`/`teardown` strings on the suite instance.
3.  **Documentation:** Several advanced features (assertions, agent eval, plugins) lack comprehensive guides.

#### Architecture Overview
- `cclab-probe` (Rust): Core logic, data structures, and assertion engine.
- `cclab-nucleus` (PyO3): Python bindings and async execution orchestration.
- `qc` module in `cclab-nucleus`: Bridges Rust and Python for testing.

#### Relevant Files
- `crates/cclab-probe/src/runner.rs`: Rust side of the runner.
- `crates/cclab-nucleus/src/qc/runner.rs`: Python-aware async runner.
- `crates/cclab-probe/src/fixtures.rs`: Fixture management.
- `crates/cclab-nucleus/src/qc/fixtures.rs`: Fixture bindings.
- `crates/cclab-probe/src/hooks.rs`: Hook definitions.
- `crates/cclab-nucleus/src/qc/hooks.rs`: Hook bindings.

#### Implementation Strategy
- **Plugin System:** Create `src/plugins.rs` in `cclab-probe` to manage a collection of plugin objects that can be registered and called.
- **Fixture DI:** Update `execute_single_test_with_gil` in `cclab-nucleus/src/qc/runner.rs` to resolve fixtures from the registry before calling the test function.
- **Async Fixtures:** Ensure the async runner properly awaits coroutines returned by fixtures.

#### Impact Analysis
- **Breaking Changes:** Minor internal changes to how `TestRunner` and `PyTestRunner` initialize and execute.
- **New Files:** `crates/cclab-probe/src/plugins.rs`, `crates/cclab-nucleus/src/qc/plugins.rs`.
- **Documentation:** New files in `crates/cclab-probe/docs/`.

#### Recommendations
- Use `algorithm` spec type for Plugin System and Fixture DI to avoid `sequence` diagram issues.
- Implement "internal plugins" first to validate the architecture.

