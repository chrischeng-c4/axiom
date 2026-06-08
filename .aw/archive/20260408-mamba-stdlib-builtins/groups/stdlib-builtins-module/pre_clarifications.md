---
change: mamba-stdlib-builtins
group: stdlib-builtins-module
date: 2026-04-09
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Should the builtins module expose functions as callable MbValue::from_func dispatchers (like functools_mod.rs) or as constants/stubs?
- **Answer**: Use MbValue::from_func with dispatch wrappers following the functools_mod.rs pattern. Each builtin function gets a dispatch_* wrapper with the (args_ptr, nargs) ABI, registered via NATIVE_FUNC_ADDRS. This allows `import builtins; builtins.print('hello')` to work correctly.

### Q2: General
- **Question**: Which builtins should be included?
- **Answer**: All builtins listed in the requirements: print, len, type, range, int, float, str, bool, list, dict, set, tuple, True, False, None, isinstance, issubclass, hasattr, getattr, setattr, id, hash, repr, abs, min, max, sum, sorted, reversed, enumerate, zip, map, filter, all, any, input, open, chr, ord, hex, oct, bin, round, pow, divmod. Map each to its existing mb_* runtime function.

