---
change: mamba-binding-tests
group: mamba-tests
date: 2026-03-22
---

# Requirements

Add comprehensive test coverage across two layers of the Mamba ecosystem, targeting 2,000+ total tests (from the current ~434) with 100% line and branch coverage.

**Layer 1 — Mamba binding crates** (user input): cclab-pg-mamba, cclab-api-mamba, cclab-runtime-mamba, cclab-agent-mamba, cclab-fetch-mamba, cclab-log-mamba, cclab-mcp-mamba each currently have 0 tests. Every mb_* function exposed by each crate must have at least one unit test covering: (a) happy path with valid inputs, and (b) error case with invalid/boundary inputs. Additionally, fix the 3 ignored tests in cclab-mamba-registry so they pass.

**Layer 2 — Mamba language runtime internals** (#1035): Fill critical per-module coverage gaps across the compiler and runtime stack.

P0 — zero coverage (implement first):
- `runtime/` (35,483 LOC): NaN-boxed MbValue round-trips for all tag types (INT, BOOL, NONE, FUNC, PTR, Float, Str, List, Dict, Set, Tuple, Bytes, Native), every builtin function (mb_print, mb_len, mb_int, mb_str, mb_float, mb_bool, mb_abs, mb_pow, mb_hash, mb_type, mb_isinstance, mb_getattr/setattr/delattr, mb_iter/next, mb_map/filter/zip/enumerate, mb_range, mb_sorted/reversed), MRO + class system (C3, __init__, super, descriptors, __slots__), GC cycle detection, module import/cache/circular, every collection method
- `lower/` (5,668 LOC): every AST→HIR lowering rule (funcdef, classdef, decorators, async/await, generators, comprehensions, match/case, try/except/finally, with, import, augmented assign, walrus)
- `resolve/` (744 LOC): scope chain (LEGB), nonlocal/global declarations, class-scope quirks, closure capture, comprehension scope (PEP 709), star import
- `hir/` (494 LOC): HIR node construction and traversal
- `driver/` (1,236 LOC): module graph cycle detection and topological sort, config merge (CLI > TOML > defaults), CompilerSession lifecycle
- `stdlib/` (15,921 LOC): every exported function in every module — priority: json, os, re, datetime, collections, pathlib, io, csv, hashlib, asyncio, math, sys, struct, random, itertools, functools

P1 — partial coverage (extend):
- `parser/` (5,829 LOC): every grammar production, error recovery, deeply nested/unicode edge cases
- `types/` (4,178 LOC): every type inference rule, generic instantiation, union narrowing, protocol structural matching
- `codegen/` (3,303 LOC): every MIR→Cranelift instruction, register allocation edge cases, NaN-boxing correctness per value type
- `lexer/` (1,469 LOC): every token type, INDENT/DEDENT mixed tabs/spaces, f-string edge cases, unicode, error recovery

**Tooling**: Use `cargo-llvm-cov` for line/branch coverage reporting. CI gate: no PR merges below 95% line coverage (ramp to 100%).

**Success metrics**: total tests ≥ 2,000; runtime direct tests ≥ 500; lower ≥ 100; resolve ≥ 50; stdlib ≥ 300; parser ≥ 280; types ≥ 100; codegen ≥ 85; lexer ≥ 40; hir+driver ≥ 60; XFAIL count = 0.
