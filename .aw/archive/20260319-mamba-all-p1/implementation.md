---
id: implementation
type: change_implementation
change_id: mamba-all-p1
---

# Implementation

## Summary

Partially implemented mamba-all-p1 (10 of 32 issues done; 22 remain open).

**#652 atexit — exit handler registration** (`atexit_mod.rs`, 127 lines):
1. LIFO handler list via `LazyLock<Mutex<Vec<String>>>`.
2. Exports: `register`, `unregister`, `_run_exitfuncs`, `_clear`, `_ncallbacks`.
3. CPython-compatible ordering: last registered, first called.

**#653 gc — garbage collector interface** (`gc_mod.rs`, 182 lines):
1. Wraps `runtime/gc.rs` cycle-detecting GC.
2. Exports: `collect`, `enable`/`disable`/`isenabled`, `get_count`, `get_threshold`/`set_threshold`, `get_stats`, `is_tracked`, `freeze`, `get_referrers`.

**#654 types — dynamic type creation utilities** (`types_mod.rs`, 199 lines):
1. Type objects for isinstance/type introspection: `FunctionType`, `MethodType`, `ModuleType`, `NoneType`, `GeneratorType`, `CoroutineType`, `AsyncGeneratorType`, `CodeType`, `BuiltinFunctionType`, `MappingProxyType`.
2. `SimpleNamespace`, `new_class`, `dynamic_class` helpers.

**#655 importlib — import machinery** (`importlib_mod.rs`, 156 lines):
1. Exports: `import_module`, `reload`, `find_spec`, `find_loader`, `invalidate_caches`.
2. `_bootstrap` submodule stub with `_find_and_load`, `_load_unlocked`.

**#656 codecs — codec registry and base classes** (`codecs_mod.rs`, 229 lines):
1. Core functions: `encode`, `decode`, `lookup`, `register`, `open`, `getincrementalencoder`/`getincrementaldecoder`.
2. Rust codec implementations: UTF-8, ASCII, Latin-1 encode/decode.
3. `CodecInfo` named tuple stub; `IncrementalEncoder`/`IncrementalDecoder` class stubs.

**#657 errno — standard errno system symbols** (`errno_mod.rs`, 209 lines):
1. 60+ POSIX errno constants with CPython-matching integer values (EPERM=1 through EHOSTUNREACH).
2. `errorcode` reverse map: int → symbol name string.

**#666 tracemalloc — trace memory allocations** (`tracemalloc_mod.rs`, 174 lines):
1. `AtomicBool`/`AtomicUsize` global state for tracing enabled/peak/current.
2. Exports: `start`, `stop`, `is_tracing`, `get_traced_memory`, `get_peak_traced_memory`, `take_snapshot`, `clear_traces`.
3. `Snapshot` struct with `compare_to`, `filter_traces`, `statistics`.

**#667 dis — bytecode disassembler** (`dis_mod.rs`, 267 lines):
1. 40+ Mamba MIR opcode constants (LOAD_CONST=100, LOAD_FAST=102, BINARY_ADD=23, etc.).
2. `Instruction` struct: `opname`, `opcode`, `arg`, `argval`, `offset`, `starts_line`, `is_jump_target`.
3. Exports: `dis`, `disassemble`, `get_instructions`, `findlinestarts`, `stack_effect`, `opmap`, `opname`.

**#668 ast — abstract syntax tree** (`ast_mod.rs`, 252 lines):
1. Exports: `parse`, `dump`, `literal_eval`, `get_docstring`, `fix_missing_locations`, `copy_location`, `walk`, `unparse`.
2. AST node class objects: `Module`, `FunctionDef`, `AsyncFunctionDef`, `ClassDef`, `Return`, `Assign`, `AugAssign`, `AnnAssign`, `For`, `While`, `If`, `With`, `Match`, `Raise`, `Try`, `Import`, `Expr`, etc.
3. `NodeVisitor`/`NodeTransformer` base class stubs.

**#669 tokenize — tokenizer for Python source** (`tokenize_mod.rs`, 346 lines):
1. Token type constants: `ENDMARKER`, `NAME`, `NUMBER`, `STRING`, `NEWLINE`, `NL`, `COMMENT`, `INDENT`, `DEDENT`, `OP`, `ERRORTOKEN` (CPython-matching values).
2. `TokenInfo` struct with `type`, `string`, `start`, `end`, `line`.
3. Exports: `generate_tokens`, `tokenize`, `detect_encoding`, `untokenize`.
4. `tok_name` forward map and `opmap` operator map.

**stdlib/mod.rs (+22 lines)**:
1. Added `pub mod` declarations for all 10 new P1 stdlib modules.
2. Added `register()` calls in `register_stdlib()` function.

**Not yet implemented (open issues)**:
- #658 selectors — high-level I/O multiplexing
- #661 ssl — TLS/SSL socket wrapper
- #662 urllib — URL handling modules
- #663 email — email handling package
- #664 multiprocessing — process-based parallelism
- #665 concurrent.futures — async execution
- #755 Exception hierarchy — Py3.12 conformance
- #756 Generator & iterator protocol — Py3.12 conformance
- #759 Data structure ops — list, dict, set, tuple, str, bytes
- #830 PEP 695 type parameter syntax — full generics support
- #831 Dict literal unpacking — {**d1, **d2, key: val}
- #832 Parenthesized with statements (PEP 617)
- #834 Exception chaining — __cause__, __context__, suppress_context
- #835 List/tuple slicing with step — a[::2], a[::-1]
- #837 Incremental compilation and module caching
- #838 REPL — interactive Mamba shell
- #840 Error diagnostics quality — rich compiler messages
- #845 Star expressions — *a, b = [1, 2, 3] extended unpacking
- #846 Global and nonlocal statements
- #847 Decorator arguments and chaining
- #848 String escape sequences — full Unicode escapes, raw strings
- #849 Class features — __slots__, __init_subclass__, properties
- #850 Async features — async for, async with, async generators

## Diff

```diff
diff --git a/crates/mamba/src/runtime/stdlib/atexit_mod.rs b/crates/mamba/src/runtime/stdlib/atexit_mod.rs
--- /dev/null
+++ b/crates/mamba/src/runtime/stdlib/atexit_mod.rs (new, 127 lines)
@@ atexit_mod.rs: exit handler registry — LIFO static list, register/unregister/run/clear/_ncallbacks; CPython-compatible API

diff --git a/crates/mamba/src/runtime/stdlib/gc_mod.rs b/crates/mamba/src/runtime/stdlib/gc_mod.rs
--- /dev/null
+++ b/crates/mamba/src/runtime/stdlib/gc_mod.rs (new, 182 lines)
@@ gc_mod.rs: GC interface wrapping runtime/gc.rs — collect, enable/disable, get_count, get_threshold/set_threshold, get_stats, is_tracked, freeze, get_referrers

diff --git a/crates/mamba/src/runtime/stdlib/types_mod.rs b/crates/mamba/src/runtime/stdlib/types_mod.rs
--- /dev/null
+++ b/crates/mamba/src/runtime/stdlib/types_mod.rs (new, 199 lines)
@@ types_mod.rs: type objects for dynamic introspection — FunctionType, MethodType, ModuleType, NoneType, GeneratorType, CoroutineType, AsyncGeneratorType, CodeType, BuiltinFunctionType, MappingProxyType, SimpleNamespace; new_class, dynamic_class

diff --git a/crates/mamba/src/runtime/stdlib/importlib_mod.rs b/crates/mamba/src/runtime/stdlib/importlib_mod.rs
--- /dev/null
+++ b/crates/mamba/src/runtime/stdlib/importlib_mod.rs (new, 156 lines)
@@ importlib_mod.rs: import machinery API — import_module, reload, find_spec, find_loader, invalidate_caches, _bootstrap submodule stub

diff --git a/crates/mamba/src/runtime/stdlib/codecs_mod.rs b/crates/mamba/src/runtime/stdlib/codecs_mod.rs
--- /dev/null
+++ b/crates/mamba/src/runtime/stdlib/codecs_mod.rs (new, 229 lines)
@@ codecs_mod.rs: codec registry — encode/decode/lookup/register routing; UTF-8, ASCII, Latin-1 Rust codec impls; CodecInfo, IncrementalEncoder/Decoder stubs

diff --git a/crates/mamba/src/runtime/stdlib/errno_mod.rs b/crates/mamba/src/runtime/stdlib/errno_mod.rs
--- /dev/null
+++ b/crates/mamba/src/runtime/stdlib/errno_mod.rs (new, 209 lines)
@@ errno_mod.rs: POSIX errno constants — EPERM through EHOSTUNREACH (60+ constants) + errorcode reverse map; CPython-compatible integer values

diff --git a/crates/mamba/src/runtime/stdlib/tracemalloc_mod.rs b/crates/mamba/src/runtime/stdlib/tracemalloc_mod.rs
--- /dev/null
+++ b/crates/mamba/src/runtime/stdlib/tracemalloc_mod.rs (new, 174 lines)
@@ tracemalloc_mod.rs: memory allocation tracing — start/stop/is_tracing, get_traced_memory/get_peak, take_snapshot; AtomicBool/AtomicUsize global state; Snapshot struct

diff --git a/crates/mamba/src/runtime/stdlib/dis_mod.rs b/crates/mamba/src/runtime/stdlib/dis_mod.rs
--- /dev/null
+++ b/crates/mamba/src/runtime/stdlib/dis_mod.rs (new, 267 lines)
@@ dis_mod.rs: MIR disassembler — 40+ Mamba MIR opcode constants, dis/disassemble/get_instructions/findlinestarts; Instruction struct with opname/opcode/arg/argval/offset; stack_effect computation

diff --git a/crates/mamba/src/runtime/stdlib/ast_mod.rs b/crates/mamba/src/runtime/stdlib/ast_mod.rs
--- /dev/null
+++ b/crates/mamba/src/runtime/stdlib/ast_mod.rs (new, 252 lines)
@@ ast_mod.rs: AST module — parse/dump/literal_eval/get_docstring/fix_missing_locations/copy_location/walk/unparse; NodeVisitor/NodeTransformer stubs; AST node class objects (Module, FunctionDef, ClassDef, etc.)

diff --git a/crates/mamba/src/runtime/stdlib/tokenize_mod.rs b/crates/mamba/src/runtime/stdlib/tokenize_mod.rs
--- /dev/null
+++ b/crates/mamba/src/runtime/stdlib/tokenize_mod.rs (new, 346 lines)
@@ tokenize_mod.rs: tokenize module — token type constants (NAME, NUMBER, STRING, OP, INDENT, DEDENT, COMMENT, NEWLINE, ENDMARKER, ERRORTOKEN), generate_tokens/tokenize/detect_encoding/untokenize; TokenInfo struct; tok_name/opmap reverse maps

diff --git a/crates/mamba/src/runtime/stdlib/mod.rs b/crates/mamba/src/runtime/stdlib/mod.rs
--- a/crates/mamba/src/runtime/stdlib/mod.rs
+++ b/crates/mamba/src/runtime/stdlib/mod.rs
@@ mod.rs: +22 lines — pub mod declarations + register() calls for 10 P1 stdlib modules
```

## Review: mamba-all-p1-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: mamba-all-p1

**Summary**: Partial implementation covering 10/33 P1 issues. All stdlib-system (atexit, gc, types, importlib, codecs, errno, tracemalloc) and stdlib-introspection (dis, ast, tokenize) modules implemented as new XXX_mod.rs files with mod.rs registration. 22 issues remain across language features, conformance, compiler infra, REPL, and IO/networking groups. Build passes, all 45 tests pass, no code issues identified.

