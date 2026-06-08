---
id: all-mamba-p0-spec
main_spec_ref: "cclab-mamba/all-mamba-p0.md"
status: retrospective
audit_date: 2026-05-08
audit_issue: "#1270"
---

# All Mamba P0 Spec

> **Audit note (2026-05-08, #1270):** this spec was authored as a forward-looking
> roadmap. Per-R-group spot-checks against current `projects/mamba/src/` confirm
> R1 / R2 / R3 / R4 are **shipped**; R5 is **partial** (mechanism shipped, the
> "100% verified" promise is open and rolled forward into the C2 conformance
> epic #1264). Inline `**Status:**` markers below capture the per-requirement
> state — the spec stays in place rather than being archived because R5 is
> still load-bearing.

## Overview
<!-- type: overview lang: markdown -->

This specification defines a comprehensive set of "P0" (highest priority) features and fixes for the Mamba project, Python's JIT/AOT compiler. The changes encompass a broad array of improvements across the entire Mamba ecosystem, focusing on enhancing Python 3.12 compliance, introducing robust compilation strategies, expanding structural language features, and ensuring robust type handling. 

The primary initiatives are:
1. **Module System & Multi-File Compilation**: Introducing support for import aliases (`import X as Y`, `from X import Y as Z`), relative imports, and full-fledged multi-file compilation. This includes building a module graph, handling cross-module type checking, and enabling separate compilation and linking.
2. **PEP 634 Structural Pattern Matching**: Implementing the full `match`/`case` feature set, including all pattern types (literal, capture, wildcard, sequence, mapping, class, OR, AS, and guards) with proper type narrowing and compilation to decision trees/Cranelift IR.
3. **BigInt Fallback for Integer Overflow**: Extending the current 48-bit NaN-boxed integer representation with a heap-allocated BigInt fallback to handle arithmetic overflows seamlessly, preserving both performance on the fast path and correctness for large integers.
4. **Benchmark Suite**: Establishing a rigorous benchmark suite comparing Mamba against CPython 3.12 and PyPy 7.3 using micro-benchmarks and real-world workloads to track performance effectively.
5. **Builtins Conformance Verification**: Validating all Python built-in functions in Mamba against CPython 3.12 behavior, transitioning existing tests from merely asserting Mamba's output to strictly verifying conformance against the reference implementation.
## Requirements
<!-- type: overview lang: markdown -->

### R1: Module System & Imports

**Status:** shipped — see evidence below.

- **R1.1**: Implement import aliases (`import X as Y`, `from X import Y as Z`).
  - Shipped: `module_alias` field on HIR import nodes; `ImportAs` resolution in `src/lower/ast_to_hir.rs` + `src/lower/hir_to_mir.rs`.
- **R1.2**: Implement relative imports (`from . import name`, `from ..mod import name`).
  - Shipped: dot-prefix relative-import parsing in `src/parser/stmt.rs` (`is_relative_start` branch + `test_from_import_relative` test).
- **R1.3**: Support multi-file compilation by building a module graph, handling `__init__.py` package structures, ordering compilation topologically, performing cross-module type checking, and enabling separate compilation with linking.
  - Shipped: see `src/runtime/module.rs` + `src/runtime/stdlib/importlib_mod.rs`.

### R2: Structural Pattern Matching (PEP 634)

**Status:** shipped — XFAIL annotations in `tests/fixtures/cpython/test_match/*.py` are now empty (`grep -n XFAIL` → 0).

- **R2.1**: Support `match`/`case` AST nodes and complete pattern syntax (literal, capture, wildcard `_`, sequence, mapping, class, OR `|`, AS, guard `if`).
  - Shipped: `Match` node in `src/hir/mod.rs`; `lower_match` in `src/lower/hir_to_mir.rs`.
- **R2.2**: Ensure proper type narrowing occurs within `case` branches.
  - Shipped: type narrowing covered by HIR's `Match` lowering and verified by `tests/fixtures/cpython/test_match/`.
- **R2.3**: Lower patterns through HIR/MIR into a decision tree and emit Cranelift IR for efficient pattern dispatch.
  - Shipped: decision-tree lowering path in `src/lower/hir_to_mir.rs`.

### R3: BigInt Fallback for NaN-Boxing

**Status:** shipped — `num-bigint` dep + dedicated `runtime/bigint_ops.rs`; INT48 overflow detection in codegen confirmed by commit `6a8b1ed0f` (`fix(mamba/codegen): detect INT48 overflow in CheckedAdd/Sub/Mul`).

- **R3.1**: Retain the fast-path 48-bit inline integer representation via NaN-boxing.
- **R3.2**: Implement an overflow check for arithmetic operations (`add`, `sub`, `mul`) that branches to a heap-allocated BigInt representation (e.g., via `num-bigint`) when 48-bit boundaries are exceeded.
- **R3.3**: Ensure seamless interoperability between inline integers and BigInts for comparison, hashing, and mixed arithmetic.

### R4: Benchmark Suite

**Status:** shipped — `projects/mamba/benches/mamba_bench.rs` + `BENCH-NOTES.md`/`NOTES-NEXT.md` track CPython/PyPy comparisons. Forward-looking perf gaps tracked under #1260, #1274, #1380, #1381, #1382 (not part of this R-group).

- **R4.1**: Develop a benchmark suite to compare Mamba (JIT/AOT) performance against CPython 3.12 and PyPy 7.3.
- **R4.2**: Include micro-benchmarks (e.g., fibonacci, nbody, spectral-norm, mandelbrot, binary-trees, fannkuch-redux) and real-world workloads (e.g., JSON processing, string manipulation).
- **R4.3**: Provide automated statistical analysis and produce comparison tables and charts for reproducible results.

### R5: Builtins Conformance (Python 3.12)

**Status:** partial — mechanism shipped (`runtime/stdlib/builtins_mod.rs` + `tests/fixtures/conformance/builtins/`), but the "100% verified" goal is open and rolled forward into epic #1264 (Builtins layer conformance). New builtins gaps are filed as standalone issues against #1264, not against this spec.

- **R5.1**: Migrate the 108 existing builtin tests from simply asserting current Mamba behavior to actively verifying conformance against CPython 3.12.
  - Shipped: conformance fixtures live under `tests/fixtures/conformance/builtins/<name>/` with byte-equivalent CPython expected output.
- **R5.2**: Achieve 100% verified compatibility for numeric (`int`, `float`, `complex`, `round`, `abs`, `pow`, `divmod`), sequence (`len`, `range`, `sorted`, `reversed`, `enumerate`, `zip`, `map`, `filter`), string (`str`, `repr`, `format`, `chr`, `ord`, `ascii`), type (`type`, `isinstance`, `issubclass`, `callable`), and all remaining built-in functions.
  - Open: tracked under epic #1264 + per-builtin issues.
## Scenarios
<!-- type: overview lang: markdown -->

### Scenario: Module Imports and Multi-file Execution
- **WHEN** a user compiles a project with multiple files involving `from . import utils` and `import math as m`
- **THEN** the compiler resolves dependencies via a module graph, ensures topological compilation order, type-checks across boundaries, and successfully links to produce an executable.

### Scenario: Structural Pattern Matching (PEP 634)
- **WHEN** the Mamba compiler encounters a `match/case` statement with complex patterns (class, sequences, guards)
- **THEN** it correctly lowers the AST into a HIR/MIR decision tree and emits optimal Cranelift IR without syntax errors, while applying type narrowing in the respective case blocks.

### Scenario: Integer Overflow
- **WHEN** an arithmetic operation results in a value exceeding the 48-bit inline integer capacity
- **THEN** the overflow check catches the condition and transparently promotes the value to a heap-allocated BigInt representation, computing the correct final result without runtime crashes or numeric truncations.

### Scenario: Benchmarking Performance
- **WHEN** a developer runs the Mamba benchmark suite against a workload like `fibonacci` or JSON processing
- **THEN** the suite automatically measures execution times, compares against installed versions of CPython 3.12 and PyPy 7.3, and produces a comparative statistical report with tables.

### Scenario: Builtins Conformance Validation
- **WHEN** the builtins conformance test suite runs
- **THEN** it executes tests for numeric, sequence, string, and type functions, explicitly verifying that Mamba's output and side-effects are strictly identical to CPython 3.12's output for all edge cases.
## Diagrams
<!-- type: overview lang: markdown -->

## API Spec
<!-- type: overview lang: markdown -->

## Test Plan
<!-- type: test_plan lang: markdown -->

- **Module System & Multi-File**: Create a test suite with multiple `.py` files inside structured directories, utilizing relative and aliased imports. Verify successful compilation, correct cross-module type inference, and accurate runtime execution.
- **Pattern Matching**: Implement an exhaustive suite of `match/case` tests covering literal, sequence, mapping, class, wildcard, and OR patterns, alongside guard conditions. Ensure type narrowing is verified.
- **Integer Overflow**: Write specific edge-case unit tests that push arithmetic operations (`+`, `-`, `*`) beyond the 48-bit limit, verifying that BigInt fallback occurs gracefully and without precision loss. Add tests mixing small inline integers and large BigInts.
- **Benchmarks**: Integrate the newly developed benchmark suite into CI. Validate that it runs cleanly, accurately measures metrics against CPython/PyPy, and outputs correct tables.
- **Builtins Conformance**: Run the updated 108 builtins tests with strict assertions against Python 3.12 reference outputs. Verify 100% pass rate for numeric, string, sequence, and type builtins.
## Changes
<!-- type: overview lang: markdown -->

- **Parser / AST**:
  - Add AST nodes and parser support for `import ... as ...` and `from ... import ... as ...`.
  - Add AST nodes and parser support for PEP 634 `match`/`case` statements and associated patterns.
- **Compiler / Frontend**:
  - Introduce module graph resolution to support multi-file builds and relative imports.
  - Implement cross-module type checking and topological sorting.
  - Implement lowering from AST to HIR/MIR decision trees for pattern matching.
- **Compiler / Backend (Cranelift)**:
  - Add instructions and lowering for BigInt promotion on integer overflow.
  - Implement pattern matching dispatch generation in Cranelift IR.
  - Add object file linking and separate compilation capabilities.
- **Runtime**:
  - Add BigInt type representation using `num-bigint` (or equivalent) in the value system.
  - Enable interoperability (arithmetic, comparison, hashing) between 48-bit inline ints and BigInts.
- **Tests & Benchmarks**:
  - Create the `benchmarks/` directory with macro/micro benchmarks and execution scripts.
  - Overhaul existing `tests/builtins/` test suite to assert results against CPython 3.12 outputs.
