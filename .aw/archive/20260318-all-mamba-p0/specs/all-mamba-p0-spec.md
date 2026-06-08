---
id: all-mamba-p0-spec
main_spec_ref: "cclab-mamba/all-mamba-p0.md"
merge_strategy: new
filled_sections: [overview, requirements, scenarios, test_plan, changes]
fill_sections: [overview, requirements, scenarios, test_plan, changes]
create_complete: true
---

# All Mamba P0 Spec

## Overview

This specification defines a comprehensive set of "P0" (highest priority) features and fixes for the Mamba project, Python's JIT/AOT compiler. The changes encompass a broad array of improvements across the entire Mamba ecosystem, focusing on enhancing Python 3.12 compliance, introducing robust compilation strategies, expanding structural language features, and ensuring robust type handling. 

The primary initiatives are:
1. **Module System & Multi-File Compilation**: Introducing support for import aliases (`import X as Y`, `from X import Y as Z`), relative imports, and full-fledged multi-file compilation. This includes building a module graph, handling cross-module type checking, and enabling separate compilation and linking.
2. **PEP 634 Structural Pattern Matching**: Implementing the full `match`/`case` feature set, including all pattern types (literal, capture, wildcard, sequence, mapping, class, OR, AS, and guards) with proper type narrowing and compilation to decision trees/Cranelift IR.
3. **BigInt Fallback for Integer Overflow**: Extending the current 48-bit NaN-boxed integer representation with a heap-allocated BigInt fallback to handle arithmetic overflows seamlessly, preserving both performance on the fast path and correctness for large integers.
4. **Benchmark Suite**: Establishing a rigorous benchmark suite comparing Mamba against CPython 3.12 and PyPy 7.3 using micro-benchmarks and real-world workloads to track performance effectively.
5. **Builtins Conformance Verification**: Validating all Python built-in functions in Mamba against CPython 3.12 behavior, transitioning existing tests from merely asserting Mamba's output to strictly verifying conformance against the reference implementation.
## Requirements

### R1: Module System & Imports
- **R1.1**: Implement import aliases (`import X as Y`, `from X import Y as Z`).
- **R1.2**: Implement relative imports (`from . import name`, `from ..mod import name`).
- **R1.3**: Support multi-file compilation by building a module graph, handling `__init__.py` package structures, ordering compilation topologically, performing cross-module type checking, and enabling separate compilation with linking.

### R2: Structural Pattern Matching (PEP 634)
- **R2.1**: Support `match`/`case` AST nodes and complete pattern syntax (literal, capture, wildcard `_`, sequence, mapping, class, OR `|`, AS, guard `if`).
- **R2.2**: Ensure proper type narrowing occurs within `case` branches.
- **R2.3**: Lower patterns through HIR/MIR into a decision tree and emit Cranelift IR for efficient pattern dispatch.

### R3: BigInt Fallback for NaN-Boxing
- **R3.1**: Retain the fast-path 48-bit inline integer representation via NaN-boxing.
- **R3.2**: Implement an overflow check for arithmetic operations (`add`, `sub`, `mul`) that branches to a heap-allocated BigInt representation (e.g., via `num-bigint`) when 48-bit boundaries are exceeded.
- **R3.3**: Ensure seamless interoperability between inline integers and BigInts for comparison, hashing, and mixed arithmetic.

### R4: Benchmark Suite
- **R4.1**: Develop a benchmark suite to compare Mamba (JIT/AOT) performance against CPython 3.12 and PyPy 7.3.
- **R4.2**: Include micro-benchmarks (e.g., fibonacci, nbody, spectral-norm, mandelbrot, binary-trees, fannkuch-redux) and real-world workloads (e.g., JSON processing, string manipulation).
- **R4.3**: Provide automated statistical analysis and produce comparison tables and charts for reproducible results.

### R5: Builtins Conformance (Python 3.12)
- **R5.1**: Migrate the 108 existing builtin tests from simply asserting current Mamba behavior to actively verifying conformance against CPython 3.12.
- **R5.2**: Achieve 100% verified compatibility for numeric (`int`, `float`, `complex`, `round`, `abs`, `pow`, `divmod`), sequence (`len`, `range`, `sorted`, `reversed`, `enumerate`, `zip`, `map`, `filter`), string (`str`, `repr`, `format`, `chr`, `ord`, `ascii`), type (`type`, `isinstance`, `issubclass`, `callable`), and all remaining built-in functions.
## Scenarios

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

## API Spec

## Test Plan

- **Module System & Multi-File**: Create a test suite with multiple `.py` files inside structured directories, utilizing relative and aliased imports. Verify successful compilation, correct cross-module type inference, and accurate runtime execution.
- **Pattern Matching**: Implement an exhaustive suite of `match/case` tests covering literal, sequence, mapping, class, wildcard, and OR patterns, alongside guard conditions. Ensure type narrowing is verified.
- **Integer Overflow**: Write specific edge-case unit tests that push arithmetic operations (`+`, `-`, `*`) beyond the 48-bit limit, verifying that BigInt fallback occurs gracefully and without precision loss. Add tests mixing small inline integers and large BigInts.
- **Benchmarks**: Integrate the newly developed benchmark suite into CI. Validate that it runs cleanly, accurately measures metrics against CPython/PyPy, and outputs correct tables.
- **Builtins Conformance**: Run the updated 108 builtins tests with strict assertions against Python 3.12 reference outputs. Verify 100% pass rate for numeric, string, sequence, and type builtins.
## Changes

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
# Reviews
