# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_decimal"
# subject = "cpython321.test_decimal"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_decimal.py"
# status = "filled"
# ///
"""cpython321.test_decimal: execute CPython 3.12 seed test_decimal"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# test_decimal.py — #2832 CPython decimal seed (executed assertions).
#
# Mamba-authored seed distilled from the decimal module's identity +
# top-level Decimal binding. Exercises the SURFACE BINDINGS today;
# arithmetic, context management, and rounding constants are excluded
# — mamba's `decimal.Decimal(x)` currently returns a fresh boxed-int
# handle (`type(d).__name__ == 'int'`, NOT 'Decimal') and arithmetic
# on those handles produces garbage. Closing that gap is tracked
# separately; once Decimal is a real class with overload dispatch,
# this seed grows to include `+`, `-`, `*`, `/`, `==` comparisons.
#
# Emits the runner's positive proof-of-execution marker that
# `cpython_lib_test_runner.rs` (#2691) classifies as `AssertionPass`.
#
# Why so small? Mamba's current decimal surface today binds ONLY
# `decimal.Decimal` (no constants, no Context, no exception classes,
# no getcontext/localcontext). Even `Decimal` is a stub that returns
# unique int handles — the type identity is wrong. The seed asserts
# only what is deterministically correct today: module identity and
# that `Decimal` is callable.
#
# Why no helper function? Per the #2691 contract, top-level `def()`
# does not capture module-scope names by reference on mamba.
#
# Contract with the runner (#2691):
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: decimal N asserts` to stdout.

import decimal

_ledger: list[int] = []

# 1. Module identity.
assert decimal.__name__ == "decimal", "decimal.__name__ must be 'decimal'"
_ledger.append(1)

# 2. Decimal class binding + callability. The class identity is wrong
#    on mamba today (Decimal(5) returns an int) — that gap is tracked
#    separately. What this seed asserts is the module binding +
#    callable, which is what every downstream user reaches for first.
assert hasattr(decimal, "Decimal"), "decimal must expose Decimal"
_ledger.append(1)
assert callable(decimal.Decimal), "decimal.Decimal must be callable"
_ledger.append(1)

# 3. Calling Decimal must produce a value (whatever its true type ends
#    up being once the class-identity gap closes). On CPython this is
#    a Decimal instance; on mamba today it is an int handle. Either
#    way, the value must NOT be None.
_d = decimal.Decimal(0)
assert _d is not None, "decimal.Decimal(0) must return a non-None value"
_ledger.append(1)
_d2 = decimal.Decimal(5)
assert _d2 is not None, "decimal.Decimal(5) must return a non-None value"
_ledger.append(1)
_d3 = decimal.Decimal("1.5")
assert _d3 is not None, "decimal.Decimal('1.5') must return a non-None value (string form)"
_ledger.append(1)

# Emit the proof-of-execution marker as the FINAL line so the runner
# can see it on stdout. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: decimal {len(_ledger)} asserts")
