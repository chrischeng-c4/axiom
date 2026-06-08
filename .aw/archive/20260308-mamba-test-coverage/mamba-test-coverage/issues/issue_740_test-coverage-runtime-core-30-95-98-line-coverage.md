---
number: 740
title: "Test coverage: Runtime core — 30% → 95–98% line coverage"
state: open
labels: [enhancement, P0, crate:mamba]
group: "runtime-core-coverage"
---

# #740 — Test coverage: Runtime core — 30% → 95–98% line coverage

## Target
Line coverage: **95–98%** (language-project standard for core runtime)

## Scope
- `src/runtime/` (excluding `stdlib/`)
- Value system, interpreter loop, builtins, memory management, GC

## Current
- ~6.1 T/KLoC
- Estimated ~30% line coverage (baseline TBD per-subsystem)

## Approach
1. Measure per-file coverage with `cargo tarpaulin`
2. Prioritize files with lowest coverage and highest complexity
3. Add unit tests for edge cases, error paths, and boundary conditions
4. Target: every public function and error path covered

## Measurement
```bash
cargo tarpaulin -p mamba --skip-clean -- runtime
```
