---
change: mamba-all-p1
group: data-structures
date: 2026-03-19
status: answered
---

# Pre-Clarifications

### Q1: General
- **Answer**: FULLY IMPLEMENTED. Slice AST parsed with 3 args (start, stop, step) in ast.rs:298-303. Parser handles all combinations in expr.rs:350-369. Runtime mb_list_slice_full() in list_ops.rs:132-176 supports positive and negative steps with proper clamping. slice_indices() helper at line 152.

### Q2: General
- **Answer**: FULLY IMPLEMENTED. Type checker (check_expr.rs:414-421) returns Ty::Any for non-integer | operands. Runtime builtins.rs:576-591 mb_bitor() routes Dict|Dict to dict_ops::mb_dict_merge(). mb_dict_merge (dict_ops.rs:320-352) creates new dict merging both, b overrides a (PEP 584 semantics). Registered in symbols.rs:200.

### Q3: General
- **Answer**: 39 methods implemented in string_ops.rs. Dispatch table at lines 848-901. Implemented: upper, lower, capitalize, title, swapcase, strip, lstrip, rstrip, find, rfind, count, startswith, endswith, replace, split, join, isdigit, isalpha, isalnum, isspace, isupper, islower, istitle, center, ljust, rjust, zfill, encode, splitlines, partition, rpartition, removeprefix, removesuffix, format, + operators (concat, repeat, getitem, slice, hash, eq, lt). Missing from CPython 47+: index, maketrans, translate, expandtabs, casefold, isidentifier, isprintable, isnumeric, isdecimal. Focus on conformance test failures.

