---
change: mamba-p0-runtime
date: 2026-02-16
---

# Clarifications

## Q1: Git Workflow
- **Question**: Which git workflow to use?
- **Answer**: in_place
- **Rationale**: Continue working on current cclab-mamba branch, consistent with prior mamba changes.

## Q2: Affected Modules
- **Question**: Which crates or paths will this change affect?
- **Answer**: crates/cclab-mamba — specifically src/runtime/ (string_ops, builtins, class, iter, exception, module, rc, symbols, new files for stdlib methods), src/codegen/ (magic method dispatch), src/lower/ (dunder lowering), and tests/
- **Rationale**: All 7 issues target the Mamba runtime layer. String/list/dict methods go in runtime, magic methods need codegen wiring, exception hierarchy extends exception.rs, builtins extend builtins.rs, file I/O adds new runtime module.

## Q3: Implementation Scope
- **Question**: How deep should each feature go? Full CPython parity or MVP subset?
- **Answer**: MVP subset — implement the most commonly used methods per type. For strings: split, join, strip, replace, startswith, endswith, upper, lower, find, count. For lists: append, extend, insert, remove, pop, sort, reverse, index, count. For dicts: get, keys, values, items, pop, update, clear. For builtins: enumerate, zip, min, max, sum, sorted, reversed, isinstance, len, range, print, input. For magic methods: __init__, __str__, __repr__, __eq__, __ne__, __lt__, __gt__, __le__, __ge__, __hash__, __len__, __getitem__, __setitem__, __contains__, __iter__, __next__, __call__, __enter__, __exit__. For exceptions: BaseException, Exception, ValueError, TypeError, KeyError, IndexError, AttributeError, RuntimeError, StopIteration, FileNotFoundError, ZeroDivisionError, ImportError, NameError, AssertionError, OSError. For file I/O: open(), read, write, readline, close, context manager.
- **Rationale**: Full CPython parity is too large for one change. MVP covers the most impactful methods that unblock real Python programs.

## Q4: Method Dispatch
- **Question**: How should method calls on built-in types be dispatched?
- **Answer**: Use type-tagged dispatch in runtime: check ObjData variant, match method name string, call corresponding Rust function. For user-defined classes, walk MRO to find dunder methods. Codegen emits mb_call_method(obj, method_name_id, args) which the runtime resolves.
- **Rationale**: Simpler than vtable approach. Type-tagged dispatch is fast for built-in types and MRO walk handles user classes correctly.

