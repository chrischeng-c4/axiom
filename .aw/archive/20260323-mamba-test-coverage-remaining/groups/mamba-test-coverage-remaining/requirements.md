---
change: mamba-test-coverage-remaining
group: mamba-test-coverage-remaining
date: 2026-03-23
---

# Requirements

Add tests across 23 source files in `crates/cclab-mamba` to reach 100% line + 100% branch coverage. No exclusion annotations (`#[cfg(coverage_not)]`, `// coverage: off`) permitted. Measure with `cargo llvm-cov --branch` after each batch.

Work is split into three batches by subsystem:

**Batch A — stdlib modules (12 files, all < 50%):**
`runtime/stdlib/argparse_mod.rs` (20%), `runtime/stdlib/platform_mod.rs` (25%), `runtime/stdlib/unittest_mod.rs` (31%), `runtime/stdlib/socket_mod.rs` (34%), `runtime/stdlib/array_mod.rs` (35%), `runtime/stdlib/errno_mod.rs` (37%), `runtime/stdlib/traceback_mod.rs` (37%), `runtime/stdlib/codecs_mod.rs` (46%), `runtime/stdlib/logging_mod.rs` (46%), `runtime/stdlib/pickle_mod.rs` (47%), `runtime/stdlib/threading_mod.rs` (49%), `runtime/stdlib/sqlite3_mod.rs` (49%).

**Batch B — core modules (3 files, all < 50%):**
`ffi/c_types.rs` (0%), `driver/mod.rs` (33%), `codegen/cranelift/mod.rs` (45%).

**Batch C — compiler pipeline (8 files, 50–78%):**
`types/check_expr.rs` (66%), `codegen/cranelift/aot.rs` (67%), `codegen/cranelift/jit.rs` (69%), `lexer/token.rs` (70%), `lower/ast_to_hir.rs` (75%), `driver/module_graph.rs` (76%), `lower/hir_to_mir.rs` (78%), `parser/expr_compound.rs` (78%).

For each file the change-spec must:
1. Enumerate every function with its branch count.
2. Identify all currently uncovered branches.
3. Define one concrete test case per uncovered branch (source string input or IR node + expected output or panic message).

Test placement: inline `#[cfg(test)]` blocks inside source files unless the file already has a dedicated integration test file. socket_mod tests must not make live network calls; threading_mod tests must be deterministic; sqlite3_mod tests use `:memory:` database.
