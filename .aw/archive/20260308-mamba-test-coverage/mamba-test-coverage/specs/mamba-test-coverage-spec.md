---
id: mamba-test-coverage-spec
main_spec_ref: "cclab-mamba/testing/test-harness.md"
merge_strategy: append
filled_sections: [overview, requirements, test_plan, changes]
fill_sections: [overview, requirements, test_plan, changes]
create_complete: true
---

# Mamba Test Coverage Spec

## Overview

Improve cclab-mamba line coverage from 30% baseline to language-project standards across all subsystems.

### Current Baseline (cargo tarpaulin)

| Subsystem | Coverage | Target |
|-----------|----------|--------|
| Parser | 93.7% | 95–98% |
| Runtime core | 50.0% | 95–98% |
| Stdlib (72 modules) | 56.9% | 80–95% |
| Lexer | 51.0% | 95–98% |
| Type checker | 70.3% | 95–98% |
| Codegen/JIT/AOT | 49.0% | 95–98% |
| FFI | 74.4% | 100% |
| HIR/MIR | ~40% | 95–98% |

### Approach

- Add inline `#[cfg(test)]` unit tests in source files for internal logic
- Add integration tests in `tests/` for cross-component behavior
- Use fixture `.py` files for parser/typecheck coverage
- Hardcode CPython 3.12 expected values for stdlib behavioral verification
- Priority: Parser+Lexer (quick wins) → FFI (safety) → Type checker → Runtime core → Stdlib → Codegen
## Requirements

### R1: Parser Coverage (93.7% → 95–98%)
- Cover uncovered branches in parser/mod.rs (78.3%), type_expr.rs (87.2%), pattern.rs (89.3%), expr.rs (93.1%)
- Add negative parse fixtures for error recovery paths
- Add positive fixtures for edge-case syntax

### R2: Lexer Coverage (51% → 95–98%)
- Cover lexer/token.rs (32.2%, 97-line gap) — all token variants, Display impls
- Cover lexer/indent.rs (95.7%) — remaining edge cases

### R3: FFI Coverage (74.4% → 100%)
- ffi/cbindgen.rs (0% → 100%) — test crate_name_from_dir, run_cbindgen error paths, generate_ffi_bindings pipeline
- ffi/c_parser.rs (79.3% → 100%) — uncovered parse branches
- ffi/stub_gen.rs (81% → 100%) — uncovered stub generation paths
- ffi/safety.rs (83.3% → 100%) — all safety guard logic
- ffi/type_map.rs (88.3% → 100%) — remaining type mappings

### R4: Type Checker Coverage (70.3% → 95–98%)
- types/protocol.rs (50%) — protocol conformance checks
- types/generic.rs (53.5%) — generic instantiation, bounds checking
- types/check_expr.rs (63%) — expression type inference
- types/check.rs (69%) — statement type checking
- types/context.rs (71.7%) — type context management
- Cover error diagnostic messages and recovery paths

### R5: Runtime Core Coverage (50% → 95–98%)
- runtime/bytes_ops.rs (0%, 131 lines) — all bytes operations
- runtime/tuple_ops.rs (26.4%) — tuple operations
- runtime/builtins.rs (27.7%, 342-line gap) — all builtin functions
- runtime/class.rs (32.7%, 333-line gap) — class system, MRO, descriptors
- runtime/list_ops.rs (34.2%) — list operations
- Add inline #[cfg(test)] modules in each source file

### R6: Stdlib Coverage (56.9% → 80–95%)
- Common modules (90–95%): sys, os, math, json, re, collections, datetime, pathlib, io, hashlib, base64, random, functools, itertools, copy, operator, struct, csv, logging, string_constants, time, decimal, fractions, contextlib, traceback, inspect, enum, dataclasses
- Edge modules (80%+): configparser, difflib, hmac, heapq, bisect, uuid, calendar, statistics, numbers, unicodedata, zlib, bz2, lzma, queue, signal, secrets, asyncio, platform, shlex, locale, abc, etc.
- Zero-test modules (dataclasses, enum, time) must get tests
- Each function: at least one positive + one negative test
- Use CPython 3.12 expected values for behavioral verification

### R7: Codegen/HIR/MIR/Resolver Coverage (49% → 95–98%)
- codegen/llvm.rs (29.8%) — test IR generation (no LLVM install needed)
- codegen/cranelift/mod.rs (42.5%) — backend compilation paths
- codegen/cranelift/jit.rs (50%) — JIT execution paths
- resolve/pass.rs + resolve/scope.rs — scope analysis, symbol binding, error paths
- HIR/MIR are mostly data structures (low executable lines) — test any executable helpers
## Diagrams

## API Spec

## Test Plan

### Testing Strategy

**Unit tests** (inline `#[cfg(test)]`):
- Each source file gets a `mod tests` with tests for internal functions
- Focus on edge cases, error paths, boundary conditions

**Integration tests** (`tests/*.rs`):
- Cross-component tests in `tests/runtime_tests.rs` (runtime)
- Fixture-based tests in `tests/fixture_tests.rs` (parser, typecheck, JIT)

**Fixture tests** (`tests/fixtures/`):
- `# RUN: parse` — syntax coverage for parser/lexer
- `# RUN: typecheck` + `# EXPECT-ERROR:` — type error paths
- `# RUN: jit` + `# EXPECT:` — codegen execution correctness

### Verification

- `cargo tarpaulin -p mamba --skip-clean` for line coverage %
- `/cclab:mamba:test-coverage` skill for test distribution and per-module detail
- All tests must pass: `cargo test -p mamba`

### Acceptance Criteria

- Parser ≥ 95% line coverage
- Lexer ≥ 95% line coverage
- FFI = 100% line coverage
- Type checker ≥ 95% line coverage
- Runtime core ≥ 95% line coverage
- Stdlib common modules ≥ 90% line coverage
- Stdlib edge modules ≥ 80% line coverage
- Codegen ≥ 95% line coverage
- All existing tests continue to pass (no regressions)
## Changes

### Modified Files (add inline tests)

- `crates/mamba/src/parser/mod.rs` — add #[cfg(test)] module
- `crates/mamba/src/parser/type_expr.rs` — add tests
- `crates/mamba/src/parser/pattern.rs` — add tests
- `crates/mamba/src/lexer/token.rs` — add tests for all token variants
- `crates/mamba/src/ffi/cbindgen.rs` — add tests
- `crates/mamba/src/ffi/c_parser.rs` — add tests
- `crates/mamba/src/ffi/stub_gen.rs` — add tests
- `crates/mamba/src/ffi/safety.rs` — add tests
- `crates/mamba/src/ffi/type_map.rs` — add tests
- `crates/mamba/src/types/protocol.rs` — add tests
- `crates/mamba/src/types/generic.rs` — add tests
- `crates/mamba/src/types/check_expr.rs` — add tests
- `crates/mamba/src/types/check.rs` — add tests
- `crates/mamba/src/runtime/bytes_ops.rs` — add tests
- `crates/mamba/src/runtime/tuple_ops.rs` — add tests
- `crates/mamba/src/runtime/builtins.rs` — add tests
- `crates/mamba/src/runtime/class.rs` — add tests
- `crates/mamba/src/runtime/list_ops.rs` — add tests
- `crates/mamba/src/runtime/stdlib/*_mod.rs` — add tests per module
- `crates/mamba/src/codegen/llvm.rs` — add tests
- `crates/mamba/src/codegen/cranelift/mod.rs` — add tests
- `crates/mamba/src/codegen/cranelift/jit.rs` — add tests
- `crates/mamba/src/resolve/pass.rs` — add tests
- `crates/mamba/src/resolve/scope.rs` — add tests

### New Files

- `crates/mamba/tests/fixtures/parse/negative/*.py` — error recovery fixtures
- `crates/mamba/tests/fixtures/typecheck/*.py` — type error fixtures
# Reviews
