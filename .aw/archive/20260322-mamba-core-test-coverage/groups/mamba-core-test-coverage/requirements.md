---
change: mamba-core-test-coverage
group: mamba-core-test-coverage
date: 2026-03-22
---

# Requirements

Bring `cclab-mamba` test coverage from 434 tests (7.1% ratio) toward 100% by adding comprehensive Rust unit/integration tests for four critically under-tested subsystems.

### Scope (P0 — zero coverage, highest priority)

1. **runtime/** (35,483 LOC, 49 existing integration tests → target 500+)
   - `value.rs`: NaN-boxed MbValue round-trips for every tag (INT, BOOL, NONE, FUNC, PTR, Float, Str, List, Dict, Set, Tuple, Bytes, Native); BigInt overflow; edge cases (NaN, Infinity, -0, max/min int)
   - `builtins.rs`: every builtin function (mb_print, mb_len, mb_int, mb_str, mb_float, mb_bool, mb_abs, mb_pow, mb_hash, mb_type, mb_isinstance, mb_getattr/setattr/delattr, mb_iter/next, mb_map/filter/zip/enumerate, mb_range, mb_sorted/reversed); type coercion matrix; error branches
   - `class.rs`: MRO C3 linearization, __init__/__new__, super(), @property/@classmethod/@staticmethod descriptors, __init_subclass__, __slots__, diamond multiple inheritance, metaclass __call__
   - `gc.rs`: cycle detection (self-ref, mutual-ref, long chain), weak refs, destructor ordering
   - `module.rs`: mb_import, mb_import_from, module cache, circular import detection, relative imports
   - `string_ops.rs`, `list_ops.rs`, `dict_ops.rs`, `set_ops.rs`, `tuple_ops.rs`: every method, empty/unicode/boundary edge cases

2. **lower/** (5,668 LOC, 0 tests → target 100+)
   - Every AST→HIR lowering rule: function def, class def, decorator application, async/await, generator/yield, all comprehension forms (list/dict/set/generator), match/case, try/except/finally, with statement, import/import-from, augmented assignment, walrus operator
   - Verify HIR output structure (node type, field values) for each input AST pattern

3. **resolve/** (744 LOC, 0 tests → target 50+)
   - Scope chain: local, enclosing (closure), global, builtin
   - `nonlocal` / `global` declarations
   - Class scope quirks (name invisibility in nested functions without nonlocal)
   - Closure variable capture semantics
   - Comprehension scope (implicit function scope per PEP 709)
   - Star import handling

4. **stdlib top-10** (subset of 15,921 LOC stdlib, 0 direct unit tests → target 150+ for these 10)
   - Priority modules: json, os, re, datetime, collections, pathlib, io, csv, hashlib, asyncio
   - Per function: happy path + error/exception cases + edge cases + type coercion
   - Tests must match Python 3.12 behavior (conformance with #750)

### Constraints
- All new tests must compile and pass with `cargo test`
- Tests must be Py3.12 conformant — test expected output against CPython 3.12 behavior
- Existing 8 XFAIL tests must remain (do not break them); conversion to passing is bonus
- No source logic changes in this change — test-only additions
- Use `cargo-llvm-cov` to measure line/branch coverage after additions
