---
change: mamba-stdlib-builtins
group: stdlib-builtins-module
date: 2026-04-09
---

# Requirements

Add native stdlib `builtins` module to Mamba. Create `crates/mamba/src/runtime/stdlib/builtins_mod.rs` that registers a `builtins` importable module exposing all existing built-in functions and constants (print, len, type, range, int, float, str, bool, list, dict, set, tuple, True, False, None, isinstance, issubclass, hasattr, getattr, setattr, id, hash, repr, abs, min, max, sum, sorted, reversed, enumerate, zip, map, filter, all, any, input, open, chr, ord, hex, oct, bin, round, pow, divmod). Follow the existing `future_mod.rs` / `main_mod.rs` pattern. Map each name to its existing runtime function pointer. In CPython `import builtins` gives access to the built-in namespace; this module replicates that.
