---
change: mamba-conformance-xfail
group: conformance-xfail-reduction
date: 2026-03-26
written_by: artifact_cli
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| cclab-mamba-fix-xfail-spec | testing | high | Xfail marker convention: `# mamba-xfail: <reason>` in .py fixture files — harness skips to avoid hangs, Conformance pipeline: parse → type-check → HIR lower → HIR-to-MIR → Cranelift JIT → stdout capture vs .expected golden file, 50 active xfail fixtures across builtins/, data_structures/, generators/, iterators/, language/, stdlib/, exceptions/, class_system/, Breakdown: 5 Cranelift verifier errors (kwargs arg count mismatch), ~3 parse errors, ~30 output mismatches, ~12 other failures, Test command: cargo test -p mamba --test conformance_tests |
| cranelift | codegen | high | R5: Function signature generation — kwargs argument count mismatch: codegen emits wrong arg count when builtin is called with kwargs (5 verifier-error xfails: list(), set(), tuple(), bytes(), print sep/end), R2: MIR instruction translation — CallExtern with keyword args must pass correct positional arg count matching runtime signature, R4: Value marshaling — keyword arguments (sep=, end=, reverse=, key=) passed as positional but counted incorrectly by signature generator, Source: codegen/cranelift/mod.rs (796 LOC) — primary target for kwargs verifier fix |
| hir-to-mir | lower | high | CallExtern lowering — kwargs are stripped before MIR emit but the stripped count must match the runtime function's arity (root of verifier errors), R5: F-string format spec lowering — relevant to string_format_xfail.py output mismatch, R2: Generator expression codegen — state machine interactions causing generator xfail edge cases, Source: lower/hir_to_mir.rs (1,728 LOC) — call arg count must align with runtime signatures |
| builtins | runtime | high | print sep/end kwargs — print_kwargs.py is one of the 5 Cranelift verifier-error xfails; runtime mb_print must accept sep/end, R2: sorted(key=, reverse=) kwargs not supported — collection_edge_cases.py, collection_builtins_edge.py, list_sort_lambda.py, R5: pow(x, y, mod) and int(x, base) not supported — numeric_edge_cases.py output mismatch, R4: chr/ord edge cases — repr_format.py output mismatch, Source: runtime/builtins.rs (891 LOC) |
| expressions | parser | medium | try/except with inline dict/list/set literals causes parse errors — 3 xfails: dict_edge_cases_xfail.py, list_edge_cases_xfail.py, set_edge_cases_xfail.py, R2: Compound expression parsing — parenthesized expressions and set literals inside except clauses, Source: parser/expr.rs, parser/expr_compound.rs |
| cranelift-jit | codegen | medium | R2: Runtime symbol table — new runtime functions for kwargs-aware builtins must be registered before JIT compilation, R3: Function finalization — Cranelift verifier errors surface here when declared sig arg count mismatches call site, Source: codegen/cranelift/jit.rs (680 LOC) |
| string-ops | runtime | medium | str.format() with keyword arguments — string_format_xfail.py output mismatch (mb_string_format kwargs path), String method edge cases contributing to output mismatch xfails, Source: runtime/string_ops.rs (989 LOC) |
| test-harness | testing | low | R1: `# mamba-xfail:` directive parsing and skip logic in conformance_tests.rs, R3: Recursive fixture discovery under tests/fixtures/conformance/ with datatest-stable harness, Timeout mechanism (10s default) relevant to generator infinite-loop xfails |

## Spec Plan

| Spec ID | Action | Main Spec Ref | Sections |
|---------|--------|---------------|----------|
| conformance-xfail-reduction-spec | modify | crates/mamba/testing/conformance.md | overview, pipeline, changes |

