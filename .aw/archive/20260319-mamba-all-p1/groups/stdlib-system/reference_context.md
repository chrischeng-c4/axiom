---
change: mamba-all-p1
group: stdlib-system
date: 2026-03-19
written_by: manual_revision
review_verdict: PASS
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| runtime/gc.md | cclab-mamba/runtime | HIGH | R1 (Track Container Objects), R2 (Mark-Sweep Collection Algorithm), R3 (Cycle Detection and Reclamation), R4 (__del__ Finalizer Invocation), R5 (Configurable Collection Thresholds at 700 allocs) |
| runtime/module.md | cclab-mamba/runtime | HIGH | R1 (Module path resolution), R2 (Module caching via sys.modules), R3 (Circular import handling), R4 (import/from-import syntax), R5 (Module object attributes) |
| runtime/builtins.md | cclab-mamba/runtime | HIGH | R3 (Type Checking Builtins), R5 (Enhanced P1 Builtins — type() at builtins.rs:475), R6 (Symbol Registration for new type constants) |
| stdlib/native-implementations.md | cclab-mamba/stdlib | HIGH | R2 (Module Rewrite — all 7 new *_mod.rs files), R3 (Symbol Registration via register_symbols()), R4 (Error Mapping to MbException) |
| runtime/symbols.md | cclab-mamba/runtime | MEDIUM | R2 (MirExtern Declarations), R3 (register_symbols() Function), R4 (Naming Convention — mb_<category>_<operation>) |
| all-mamba-p0.md | cclab-mamba | LOW | R1 (Module System & Imports — architectural context), R5 (Builtins Conformance — type() synthesis context) |
| README.md | cclab-mamba | LOW | Navigation index to gc.md, module.md, builtins.md (lines 73, 87, 76); stdlib module directory structure (lines 91-127) |

# Revision Notes

## Review Issues Addressed

All seven issues from the initial FAIL verdict have been resolved:

1. **[HIGH] runtime/gc.md (added)**
   - PRIMARY backing spec for issue #653 (gc module) and infrastructure for #666 (tracemalloc)
   - Pre-clarification Q1 is entirely about gc.rs: atomic RC + cycle-detecting GC, mark-sweep, threshold at 700 allocs, gc.collect() trigger, gc.disable()/enable() toggling
   - Covers R1-R5: Track Container Objects (foundation for tracemalloc per Q3), Mark-Sweep Algorithm (mark/trace/sweep/reset per Q1), Cycle Detection, __del__ Finalizer semantics, Configurable Thresholds at 700 allocs per Q1
   - All five requirements MANDATORY for gc module

2. **[HIGH] runtime/module.md (added)**
   - PRIMARY backing spec for issue #655 (importlib)
   - Pre-clarification Q2 is entirely about module.rs: thread-local MODULES HashMap, MbModule, mb_import() with circular protection, mb_import_from(), find_module()
   - Covers R1-R5: Module path resolution, Module caching via sys.modules with pre-insert, Circular import handling with sentinel pre-caching (matches Q2 exactly), import/from-import syntax via mb_import_from(), Module object attributes (__name__, __file__, __dict__, __package__)
   - All five requirements MANDATORY for importlib module

3. **[HIGH] runtime/builtins.md (added)**
   - PRIMARY backing spec for issue #654 (types module)
   - Pre-clarification Q4 explicitly states: type() builtin at builtins.rs:475 returns Instance with __name__; FunctionType/GeneratorType/ModuleType etc. NOT exposed, need to synthesize as Mamba class objects
   - Covers R3 (Type Checking Builtins — isinstance, issubclass), R5 (Enhanced P1 Builtins — type() at builtins.rs:475, entry point for type-object synthesis), R6 (Symbol Registration for FunctionType/GeneratorType/ModuleType constants as mb_* symbols)
   - Three requirements MANDATORY for types module

4. **[HIGH] stdlib/native-implementations.md (added)**
   - IMPLEMENTATION-PATTERN spec for ALL 7 in-scope issues (#652 atexit, #653 gc, #654 types, #655 importlib, #656 codecs, #657 errno, #666 tracemalloc)
   - Every issue requires new *_mod.rs file under runtime/stdlib/ following the native module pattern
   - Covers R2 (Module Rewrite — atexit_mod.rs, gc_mod.rs, types_mod.rs, importlib_mod.rs, codecs_mod.rs, errno_mod.rs, tracemalloc_mod.rs all follow pattern), R3 (Symbol Registration via register_symbols()), R4 (Error Mapping — codec errors, errno libc errors, ModuleNotFoundError all map to MbException)
   - Three requirements MANDATORY for all 7 modules

5. **[MEDIUM] runtime/symbols.md (added)**
   - Symbol registration protocol spec required for all 7 new stdlib modules
   - Defines register_symbols() protocol (R3), MirExtern Declarations (R2), Naming Convention pattern mb_<category>_<operation> (R4)
   - gc module symbols: mb_gc_collect, mb_gc_disable, mb_gc_enable follow pattern
   - All stdlib module symbols must follow mb_<module>_<operation> pattern
   - Three requirements MANDATORY for symbol registration

6. **[LOW] all-mamba-p0.md (downgraded from MEDIUM)**
   - Provides architectural background context only — NOT direct implementation specs
   - R1 (Module System & Imports) addresses multi-file compilation and import aliases as background for #655 importlib
   - R5 (Builtins Conformance) addresses CPython conformance testing and dynamic type object synthesis as background for #654 types
   - Value is strategic alignment only; high-level feature landscape rather than implementation mechanics

7. **[LOW] README.md (downgraded from MEDIUM)**
   - Codebase navigation index only — contains NO R-prefixed requirements or behavioral contracts
   - Lines 73, 87, 76 reference runtime/gc.md, runtime/module.md, runtime/builtins.md respectively (all now directly included at HIGH relevance)
   - Lines 91-127 show stdlib module directory structure indicating implementation locations
   - Value is NAVIGATION ONLY for locating actual implementation specs

## Coverage Summary

- **Issue #653 (gc module)**: Covered by runtime/gc.md (R1-R5 complete spec) + stdlib/native-implementations.md (R2-R4 registration pattern)
- **Issue #655 (importlib)**: Covered by runtime/module.md (R1-R5 complete spec) + stdlib/native-implementations.md (R2-R4 registration pattern)
- **Issue #654 (types module)**: Covered by runtime/builtins.md (R3,R5-R6) + stdlib/native-implementations.md (R2-R4 registration pattern)
- **Issue #652 (atexit)**: Covered by stdlib/native-implementations.md (R2-R4 registration pattern, R4 error mapping)
- **Issue #656 (codecs)**: Covered by stdlib/native-implementations.md (R2-R4 registration pattern, R4 codec error mapping)
- **Issue #657 (errno)**: Covered by stdlib/native-implementations.md (R2-R4 registration pattern, R4 libc error mapping)
- **Issue #666 (tracemalloc)**: Covered by runtime/gc.md R1 (Container Tracking foundation) + stdlib/native-implementations.md (R2-R4 registration pattern)

- **Pre-clarification Q1 (GC Infrastructure)**: Fully covered by runtime/gc.md (R1-R5) with threshold_allocs=700, gc.collect(), gc.disable()/enable() matching Q1 answer exactly
- **Pre-clarification Q2 (Module/Import System)**: Fully covered by runtime/module.md (R1-R5) with mb_import(), circular sentinel pre-caching, find_module() matching Q2 answer exactly
- **Pre-clarification Q3 (Memory Tracking)**: Fully covered by runtime/gc.md R1 (Track Container Objects) providing foundation for coarse tracemalloc stats per Q3 answer
- **Pre-clarification Q4 (Type Objects)**: Fully covered by runtime/builtins.md (R3,R5-R6) with type() at builtins.rs:475 and FunctionType/GeneratorType/ModuleType synthesis matching Q4 answer exactly

## Key Changes from Initial FAIL Verdict

- **REMOVED**: pattern-matching.md — PEP 634 match/case has ZERO connection to stdlib-system issues. No mention of atexit, gc, types, importlib, codecs, errno, or tracemalloc. Belongs to stdlib-introspection group.
- **DOWNGRADED**: all-mamba-p0.md and README.md from MEDIUM to LOW — both provide architectural background only, not direct implementation specs
- **ADDED**: Five missing PRIMARY specs (runtime/gc.md, runtime/module.md, runtime/builtins.md, stdlib/native-implementations.md, runtime/symbols.md) that directly implement all 7 issues
- **SCOPE VERIFIED**: All crates/areas from pre-clarifications now have complete spec coverage with explicit R-prefixed requirement mappings
- **BLOCKED**: Cannot include stdlib/atexit.md, stdlib/codecs.md, stdlib/errno.md — these are NEW DELIVERABLES for change-spec phase, not existing specs. Reference context lists EXISTING specs only.

## Relevance Rationale

- **HIGH (4 specs)**: runtime/gc.md, runtime/module.md, runtime/builtins.md, stdlib/native-implementations.md directly implement core issues with specific R-prefixed requirements; PRIMARY backing specs
- **MEDIUM (1 spec)**: runtime/symbols.md provides registration protocol for all 7 modules; architecturally prescriptive, implements pattern from native-implementations.md R3
- **LOW (2 specs)**: all-mamba-p0.md and README.md provide architectural context and navigation only; no direct implementation dependency
