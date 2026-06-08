---
number: 1035
title: "test(mamba): per-module test coverage gaps — lower, resolve, runtime severely undertested"
state: open
labels: [enhancement, P1, crate:mamba]
group: "mamba-tests"
---

# #1035 — test(mamba): per-module test coverage gaps — lower, resolve, runtime severely undertested

## Context

Audit of Mamba test coverage revealed critical gaps. Total: **434 tests** (253 unit + 178 fixture + 3 doc) covering **61,943 LOC** source. Test:source ratio is **7.1%**.

**Target: 100% test coverage.** Mamba is a language runtime — every code path must be verified. There is no acceptable gap for compiler/runtime infrastructure.

Related: #750 (Py3.12 conformance tracking), #1028 (language blockers)

## Per-Module Coverage

| Module | Source LOC | Direct Tests | Current State | Target |
|--------|----------|--------------|---------------|--------|
| **runtime** | 35,483 (57%) | 49 unit | **Critical** | 100% — every builtin, every value tag, every GC path |
| **stdlib** | 15,921 (in runtime) | 0 unit | **Zero** | 100% — every module function, every error branch |
| **lower** (AST→HIR) | 5,668 | 0 | **Zero** | 100% — every lowering rule, every node type |
| **resolve** (name resolution) | 744 | 0 | **Zero** | 100% — every scope rule, every binding kind |
| **parser** | 5,829 | 17 unit + 186 fixture | Partial | 100% — every grammar production |
| **types** | 4,178 | 45 unit | Partial | 100% — every type rule, every inference path |
| **codegen** | 3,303 | 35 jit | Partial | 100% — every instruction emission |
| **lexer** | 1,469 | 12 unit | Basic | 100% — every token type, every edge case |
| **driver** | 1,236 | 0 direct | **Zero** | 100% — module graph, config merge |
| **config** | 321 | ~8 unit | OK | 100% |
| **hir** | 494 | 0 direct | **Zero** | 100% |

## Priority Order

### P0 — Zero Coverage (blocking correctness)

**1. runtime/** (35,483 LOC → need ~500+ tests)
- `value.rs` — NaN-boxed MbValue: round-trip for ALL tag types (INT, BOOL, NONE, FUNC, PTR, Float, Str, List, Dict, Set, Tuple, Bytes, Native), overflow to BigInt, edge cases (NaN, Infinity, -0)
- `builtins.rs` — Every builtin function: `mb_print`, `mb_len`, `mb_int`, `mb_str`, `mb_float`, `mb_bool`, `mb_abs`, `mb_pow`, `mb_hash`, `mb_type`, `mb_isinstance`, `mb_getattr`, `mb_setattr`, `mb_delattr`, `mb_iter`, `mb_next`, `mb_map`, `mb_filter`, `mb_zip`, `mb_enumerate`, `mb_range`, `mb_sorted`, `mb_reversed`, all comparison/arithmetic operators, type coercion matrix
- `class.rs` — MRO (C3 linearization), `__init__`, `__new__`, `super()`, descriptors (`@property`, `@classmethod`, `@staticmethod`), `__init_subclass__`, `__slots__`, multiple inheritance diamond, metaclass `__call__`
- `gc.rs` — Cycle detection: self-ref, mutual ref, long chain, weak refs, destructor ordering
- `module.rs` — `mb_import`, `mb_import_from`, module cache, circular import handling, `__init__.py` packages, relative imports
- `string_ops.rs` — Every string method with edge cases (empty string, unicode, encoding)
- `list_ops.rs`, `dict_ops.rs`, `set_ops.rs`, `tuple_ops.rs` — Every collection method, boundary conditions

**2. lower/** (5,668 LOC → need ~100+ tests)
- Every AST→HIR lowering rule: function def, class def, decorator application, async/await, generator/yield, comprehension (list/dict/set/generator), match/case, try/except/finally, with statement, import statement, augmented assignment, walrus operator
- Verify HIR output structure for each input AST pattern

**3. resolve/** (744 LOC → need ~50+ tests)
- Scope chain: local, enclosing, global, builtin
- `nonlocal` / `global` declarations
- Class scope quirks (name not visible in nested functions without nonlocal)
- Closure variable capture
- Comprehension scope (implicit function scope per PEP 709)
- Star import handling

**4. hir/** (494 LOC → need ~30+ tests)
- HIR node construction and traversal
- HIR → MIR lowering interface

**5. driver/** (1,236 LOC → need ~30+ tests)
- Module graph: cycle detection, topological sort
- Config merge: CLI > TOML > defaults
- CompilerSession lifecycle

### P1 — Partial Coverage (needs completion)

**6. stdlib/** (15,921 LOC → need ~300+ tests)
- Every module, every exported function
- Top priority: json, os, re, datetime, collections, pathlib, io, csv, hashlib, asyncio, math, sys, struct, random, itertools, functools
- Each function: happy path + error cases + edge cases + type coercion

**7. parser/** (5,829 LOC → need ~80+ more tests)
- Every grammar production rule
- Error recovery paths
- Edge cases: deeply nested expressions, max recursion, unicode identifiers

**8. types/** (4,178 LOC → need ~60+ more tests)
- Every type inference rule
- Generic instantiation
- Union type narrowing
- Protocol structural matching

**9. codegen/** (3,303 LOC → need ~50+ more tests)
- Every MIR→Cranelift instruction
- Register allocation edge cases
- NaN-boxing correctness for every value type in compiled code

**10. lexer/** (1,469 LOC → need ~30+ more tests)
- Every token type
- INDENT/DEDENT: mixed tabs/spaces, edge cases
- f-string: nested quotes, backslash, multiline
- Unicode: identifiers, string literals
- Error recovery: unterminated strings, invalid escape sequences

## Success Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Total tests | ~434 | **2,000+** |
| runtime direct tests | 49 | **500+** |
| lower direct tests | 0 | **100+** |
| resolve direct tests | 0 | **50+** |
| stdlib unit tests | 0 | **300+** |
| parser tests | 203 | **280+** |
| types tests | 45 | **100+** |
| codegen tests | 35 | **85+** |
| lexer tests | 12 | **40+** |
| hir/driver tests | 0 | **60+** |
| XFAIL count | 8 | **0** |
| Line coverage | unknown | **100%** |
| Branch coverage | unknown | **100%** |

## Tooling

- Use `cargo-llvm-cov` for line/branch coverage reporting
- CI gate: no PR merges below 95% line coverage (ramp to 100%)
- Coverage report per module in CI artifacts
