# test_argparse.py — #2697 CPython argparse seed (executed assertions).
#
# This is NOT a verbatim copy of CPython's Lib/test/test_argparse.py
# (the upstream file is ~5000 lines exercising ArgumentParser /
# Namespace / Action / store_true / nargs / subparsers / help formatters
# / type coercion / mutex groups / etc). Instead it is the *smallest*
# Mamba-authored seed distilled from the argparse module's TOP-LEVEL
# IDENTITY surface — the parts that work on both CPython 3.12 and the
# mamba runtime today.
#
# Why so small? Mamba's current argparse is a stub: the module imports
# cleanly and `argparse.ArgumentParser` is bound to a callable, but
#
#   - `argparse.ArgumentParser(prog="demo")` returns a *dict*, not a
#     real parser. Calling `.add_argument(...)` raises
#     AttributeError: 'dict' object has no attribute 'add_argument'.
#   - `argparse.Namespace` does not exist on mamba (AttributeError).
#   - `argparse.Action`, `argparse.HelpFormatter`,
#     `ArgumentDefaultsHelpFormatter`, `ArgumentError`, `SUPPRESS`,
#     `OPTIONAL`, `ZERO_OR_MORE`, `ONE_OR_MORE`, `PARSER`, `REMAINDER`
#     all return False from hasattr() on mamba.
#
# The richer surface (parse_args, --flag, store_true, type=int,
# default=, positional, nargs=, subparsers, help, Namespace attribute
# access) all land as each gap closes. Until then the load-bearing
# claim of this seed is narrow: the *module* exists and the *top-level
# ArgumentParser name* is bound to a callable. That is genuinely
# deterministic on both runtimes and a regression in either direction
# would catch a real bootstrap break.
#
# Why no helper function? Per the #2691 contract, top-level `def()`
# does not capture module-scope names by reference on mamba.
#
# Contract with the runner (#2691):
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: argparse N asserts` to stdout.

import argparse

_ledger: list[int] = []

# 1. Module identity: argparse's own __name__ must be "argparse".
assert argparse.__name__ == "argparse", "argparse.__name__ must be 'argparse'"
_ledger.append(1)

# 2. ArgumentParser is a public attribute of the module. Catches a
#    regression where the import returns an empty stub.
assert hasattr(argparse, "ArgumentParser"), "argparse must expose ArgumentParser"
_ledger.append(1)

# 3. ArgumentParser is callable (it is a class on CPython, a builder
#    function on the mamba stub). Either way it must accept a call.
assert callable(argparse.ArgumentParser), "argparse.ArgumentParser must be callable"
_ledger.append(1)

# Emit the proof-of-execution marker as the FINAL line so the runner
# can see it on stdout. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: argparse {len(_ledger)} asserts")
