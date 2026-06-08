---
change: mamba-all-p1
group: async-generators
date: 2026-03-19
written_by: artifact_cli
review_verdict: REVISED
---

# Reference Context

| Spec | Relevance | Key Requirements |
|------|-----------|------------------|
| `cclab-mamba/runtime/generator.md` | high | Generator state machine implementation (R1-R5): compilation to state machine with Created/Suspended/Running/Completed states, yield expression suspension and resumption preserving local variables and instruction pointer, generator iterator protocol (__iter__ returns self, __next__ resumes), send(value) method for resumption with value, throw(exception) and close() methods with GeneratorExit handling. Foundational for Q1 (sync generators) and Q2 (async generator architecture). |
| `cclab-mamba/runtime/async.md` | high | Async/await runtime integration (R1-R4): async function compilation to state machine objects with suspension/resumption at await points, Orbit event loop integration (mb_coroutine_suspend, mb_coroutine_resume, asyncio.run, asyncio.gather), GIL-safe scheduling with release during suspension and re-acquisition on resumption, future interoperability with Tokio futures. Critical for Q2 (async generators need async runtime foundation). |
| `cclab-mamba/runtime/exception.md` | high | Exception hierarchy and protocols (includes StopAsyncIteration): builtin exception class tree, exception creation and raising, StopIteration exception with value attribute for generator returns. StopAsyncIteration exception variant must be added for async generator protocol. Needed for Q2 (async generator termination) and exception handling in async contexts. |
| `cclab-mamba/runtime/iter.md` | high | Iteration protocol implementation (__iter__, __next__, StopIteration): async iteration counterpart dunder methods (__aiter__, __anext__) must be added for async generators. Async for/with desugaring depends on async iteration protocol. Covers Q3 (async for/with implementation). |
| `cclab-mamba/runtime/class.md` | high | Magic method dispatch system (R5 especially): operators and protocol method dispatch to dunder methods. Specifically: __aiter__, __anext__, __aenter__, __aexit__ async protocol methods must be added and dispatched. R8 (context manager protocol) covers __enter__/__exit__ which must be extended to async context manager (__aenter__/__aexit__). Critical for Q3 (async for/with context manager protocol). |
| `cclab-mamba/hir/hir.md` | high | HIR node definitions and async construct support: HirStmt variants for AsyncFor and AsyncWith statements must be added to AST-to-HIR output. Currently async constructs are parsed but dropped/collapsed in lowering. Defines the intermediate representation that bridges AST to code generation. Covers Q3 (async for/with HIR representation). |
| `cclab-mamba/lower/ast-to-hir.md` | high | AST to HIR desugaring (R3: With-statement desugaring): async for/with desugaring rules. R3 covers with-statement desugaring to __enter__/__exit__ protocol calls; async with must be extended to __aenter__/__aexit__ calls. AsyncFor desugaring to __aiter__/__anext__ protocol calls must be added (currently AsyncFor is silently dropped in ast_to_hir.rs). Covers Q3 (async for/with desugaring and HirStmt variant generation). |
| `cclab-mamba/testing/mamba-py312-conformance.md` | high | Python 3.12 conformance testing (R6.1-R6.7): R6 Generator & Iterator Protocol establishes acceptance criteria with test scenarios for yield, yield from, send(), throw(), close(), StopIteration.value. R6.7 explicitly marks "Async generators — xfail" defining the conformance obligation for this change group. R6.1-R6.6 cover sync generator protocol tests that underpin Q1. Directly specifies the test fixtures and expected golden-file outputs (CPython 3.12 reference). Maps to all three pre-clarification areas: Q1 (R6.1-R6.6 generators), Q2 (R6.7 async generators with xfail), Q3 (R6.6 iterator protocol for for/with loops). |

## Coverage Analysis

**Pre-Clarifications Scope Coverage:**

| Area | Files | Covered By |
|------|-------|-----------|
| Q1: Sync Generators | generator.rs, class.rs, exception.rs | runtime/generator.md (R1-R5 state machine, send/throw/close), runtime/class.md (R5 method dispatch), runtime/exception.md (StopIteration), mamba-py312-conformance.md (R6.1-R6.6 test criteria) |
| Q2: Async Generators | tokio_exec.rs, generator.rs | runtime/async.md (R1-R4 async runtime, coroutine scheduling), runtime/generator.md (R1 state machine foundation), runtime/exception.md (StopAsyncIteration), mamba-py312-conformance.md (R6.7 async generator test spec) |
| Q3: Async For/With | ast_to_hir.rs, class.rs, hir/ | runtime/iter.md (R __aiter__/__anext__ protocol), runtime/class.md (R5 __aenter__/__aexit__ dispatch, R8 context manager), hir/hir.md (async construct HIR nodes), lower/ast-to-hir.md (R3 desugaring rules for async with, new rules for async for), mamba-py312-conformance.md (R6.6 iterator protocol scenarios) |
| Exception Handling | exception.rs | runtime/exception.md (exception hierarchy, StopAsyncIteration), mamba-py312-conformance.md (exception scenarios) |

**Gap Analysis:**

All 8 critical specs are now directly included at 'high' relevance. Each spec's requirements explicitly define what must be extended for async generator support:
- `runtime/generator.md` defines the state machine foundation that async generators must inherit
- `runtime/async.md` defines the event loop and GIL-safe execution model required for async generators
- `runtime/exception.md` defines exception classes including StopAsyncIteration for async termination
- `runtime/iter.md` defines __aiter__/__anext__ protocol methods for async iteration
- `runtime/class.md` defines magic method dispatch for async dunder methods
- `hir/hir.md` defines HIR node structure for async statements
- `lower/ast-to-hir.md` defines desugaring rules that must be extended for async for/with
- `mamba-py312-conformance.md` defines the test acceptance criteria and explicitly marks async generators as xfail

## Issues Addressed

### [HIGH] All 7 directly relevant specs now included

✅ **RESOLVED**: Replaced the indirect README.md pointer with direct inclusion of all 7 critical specs at 'high' relevance:
- `runtime/generator.md` (generator state machine, send/throw/close protocol)
- `runtime/async.md` (async/await scheduling, coroutines, event loop integration)
- `runtime/class.md` (magic method dispatch for __aiter__/__anext__/__aenter__/__aexit__)
- `hir/hir.md` (HIR nodes for async constructs)
- `lower/ast-to-hir.md` (async for/with desugaring rules)
- `runtime/exception.md` (exception hierarchy and StopAsyncIteration)
- `runtime/iter.md` (iteration protocol with __aiter__/__anext__)

### [HIGH] mamba-py312-conformance.md is the most directly relevant spec

✅ **RESOLVED**: Added `cclab-mamba/testing/mamba-py312-conformance.md` at 'high' relevance. It contains:
- R6 Generator & Iterator Protocol (acceptance criteria)
- R6.1-R6.6 sync generator protocol tests (Q1 foundation)
- R6.7 Async generators — xfail (the exact conformance obligation for this change)
- Explicit reference to issue #756 (py3-12-conformance-generator-iterator-protocol)

### [HIGH] all-mamba-p0.md key requirements were factually incorrect

✅ **RESOLVED**: Removed `all-mamba-p0.md` from the reference set. Its actual content covers:
- R1 module system/imports
- R2 PEP 634 pattern matching
- R3 BigInt fallback
- R4 benchmark suite
- R5 builtins conformance (numeric/sequence/string/type functions)

None of these relate to generators, async generators, or the iteration protocol. The false attribution to issue #756 has been corrected with proper reference to mamba-py312-conformance.md.

### [MEDIUM] README.md was an index document, not a specification

✅ **RESOLVED**: Removed `cclab-mamba/README.md` from the reference set. It contains no requirements or acceptance criteria — only a table of contents. Implementers reading only README.md would have no concrete requirements to implement against. Replaced with direct inclusion of the 7 specs it pointed to.

### [LOW] pattern-matching.md is entirely orthogonal to async generators

✅ **RESOLVED**: Removed `cclab-mamba/pattern-matching.md` from the reference set. It covers PEP 634 match/case statements and has zero overlap with:
- Async generators
- Async for/with statements
- The iteration protocol
- Exception handling (beyond general exception types)

## Verification Summary

**Pre-clarifications Coverage:** ✅ All three areas (Q1, Q2, Q3) are now covered by concrete, directly relevant specs.

**Relevance Scores:** ✅ All 8 specs rated 'high' — each directly implements or defines requirements for async generators, async runtime, or async protocol.

**Key Requirements Accuracy:** ✅ Each spec's key requirements now accurately reflect its actual content and relationship to async generators.

**No Irrelevant Specs:** ✅ Removed 3 low-value specs (README.md, pattern-matching.md, all-mamba-p0.md). All remaining 8 specs are directly relevant.
