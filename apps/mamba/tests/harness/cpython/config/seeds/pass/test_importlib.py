# test_importlib.py — #2691 CPython importlib seed (executed assertions).
#
# This is NOT a verbatim copy of CPython's Lib/test/test_importlib/
# (that test package pulls in dozens of files and the full unittest
# discovery surface). Instead it is the *smallest* Mamba-authored
# seed distilled from the CPython importlib smoke surface: it
# asserts the module identity invariant
# (`importlib.__name__ == "importlib"`) with raw `assert` statements
# and emits a positive proof-of-execution marker that the runner
# (`cpython_lib_test_runner.rs`, #2691) classifies as
# `AssertionPass` — not `ImportPass` or `Stub`.
#
# Why so small? Mamba's import system currently presents importlib
# as a module-shaped surface with `__name__` populated but most
# concrete APIs (`importlib.import_module`, `importlib.util.find_spec`)
# absent. The shape that DOES work today is the module-identity
# invariant; richer asserts will flip from Fail to AssertionPass
# automatically as mamba grows the importlib API — at which point
# the seed should be expanded in the same commit that closes the
# gap. Until then this is the load-bearing seed that proves the
# AssertionPass classification path itself works end-to-end.
#
# Why no helper function? Mamba's current runtime has a gap where
# a top-level def() does not capture module-scope names by
# reference (lookup yields a stale empty value). Asserts are
# therefore inlined at module top-level so every check executes
# in the same scope the counter ledger lives in.
#
# Contract with the runner (#2691):
#   - Each `assert` runs at top level. Inversion (e.g. flipping
#     `assert importlib.__name__ == "importlib"` to `... == "not"`)
#     raises AssertionError → non-zero exit → runner classifies
#     as `Fail`, never silently passes.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: importlib N asserts` to stdout. The
#     runner sees the marker and classifies as `AssertionPass`.
#   - The seed does NOT touch the user's site-packages.

import importlib

_ledger: list[int] = []

# 1. Module identity: importlib's own __name__ must be "importlib".
#    This is the canonical CPython invariant (it's the first thing
#    checked by `Lib/test/test_importlib/test_api.py` and similar).
#    Inversion fails the runner.
assert importlib.__name__ == "importlib", "importlib.__name__ must be 'importlib'"
_ledger.append(1)

# 2. __name__ is a real `str` (not None, not a numeric handle, not
#    a placeholder). Catches a class of bootstrap regressions where
#    the module is loaded but identity metadata is mis-typed.
assert isinstance(importlib.__name__, str), "importlib.__name__ must be a str"
_ledger.append(1)

# 3. The __name__ string is non-empty. Distinguishes "stubbed-with-
#    empty-string" from "stubbed-with-real-value".
assert len(importlib.__name__) > 0, "importlib.__name__ must be non-empty"
_ledger.append(1)

# Emit the proof-of-execution marker as the FINAL line so the runner
# can see it on stdout. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: importlib {len(_ledger)} asserts")
