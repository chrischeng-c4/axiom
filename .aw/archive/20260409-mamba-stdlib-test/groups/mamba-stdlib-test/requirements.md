---
change: mamba-stdlib-test
group: mamba-stdlib-test
date: 2026-04-09
---

# Requirements

Add a native stdlib `test` module to crates/cclab-mamba. Create `crates/mamba/src/runtime/stdlib/test_mod.rs` following the pattern of existing stdlib modules (e.g. builtins_mod.rs). The module registers as `test` and exports: TestCase class with assertEqual, assertTrue, assertFalse, assertRaises methods; main() function as test runner. This is a simplified version of CPython's unittest/test support, sufficient for basic test scripts. Must pass all existing cclab-mamba tests with no regressions.
