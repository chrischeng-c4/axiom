# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_float_divzero_silent_inf"
# subject = "cpython321.lang_float_divzero_silent_inf"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_float_divzero_silent_inf.py"
# status = "filled"
# ///
"""cpython321.lang_float_divzero_silent_inf: execute CPython 3.12 seed lang_float_divzero_silent_inf"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Spec seed for CPython ZeroDivisionError contract on the
# float-divisor corners that mamba silently returns IEEE-754
# `inf` / `-inf` / `nan` / `None` from. Surface: CPython rejects
# every float division/modulo/divmod where the divisor is zero —
# `f/0.0`, `f%0.0`, `divmod(f, 0.0)` — with ZeroDivisionError
# regardless of whether the divisor is `0.0` or `-0.0`. Python's
# `float` type chooses to raise rather than return the IEEE-754
# `inf` / `nan` values, because in pure Python those values are
# only producible through explicit construction (`float('inf')`).
# Mamba instead returns `inf` for `positive / 0.0`, `-inf` for
# `negative / 0.0` (and `f / -0.0`), `nan` for `0.0 / 0.0` and
# `f % 0.0`, and `None` for `divmod(f, 0.0)`. Code that does
# `if denominator != 0: ratio = x/denominator else: ratio = 0`
# silently turns the wrong branch when the denominator was
# already zero. This is a high-impact silent-coercion class
# bridging the int-zerodiv probes already pinned by other specs
# (`lang_math_domain_silent`) into the float-division regime.
#
# Probes (every form CPython rejects, mamba silently returns IEEE):
#   • 1.0 / 0.0                → mamba: inf            (ZeroDivisionError)
#   • -1.0 / 0.0               → mamba: -inf           (ZeroDivisionError)
#   • 0.0 / 0.0                → mamba: nan            (ZeroDivisionError)
#   • 1.0 / -0.0               → mamba: -inf           (ZeroDivisionError)
#   • 1.0 % 0.0                → mamba: nan            (ZeroDivisionError)
#   • divmod(1.0, 0.0)         → mamba: None           (ZeroDivisionError)
#   • divmod(0.0, 0.0)         → mamba: None           (ZeroDivisionError)
#   • divmod(2.5, 0.0)         → mamba: None           (ZeroDivisionError)
#   • divmod(-1.5, 0.0)        → mamba: None           (ZeroDivisionError)
#
# CPython contract (uniform for every signed float zero):
#   f / 0.0  /  f / -0.0       → ZeroDivisionError("float division
#                                                  by zero");
#   f % 0.0                    → ZeroDivisionError("float modulo");
#   divmod(f, 0.0)             → ZeroDivisionError("float divmod()").
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so
# the runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_zero_f: Any = 0.0
_negzero_f: Any = -0.0
_one_f: Any = 1.0
_negone_f: Any = -1.0
_twohalf_f: Any = 2.5
_neg_oneandhalf_f: Any = -1.5

# 1.0 / 0.0
try:
    _ = _one_f / _zero_f
    raise AssertionError("1.0 / 0.0 must raise ZeroDivisionError")
except ZeroDivisionError:
    _ledger.append(1)

# -1.0 / 0.0
try:
    _ = _negone_f / _zero_f
    raise AssertionError("-1.0 / 0.0 must raise ZeroDivisionError")
except ZeroDivisionError:
    _ledger.append(1)

# 0.0 / 0.0
try:
    _ = _zero_f / _zero_f
    raise AssertionError("0.0 / 0.0 must raise ZeroDivisionError")
except ZeroDivisionError:
    _ledger.append(1)

# 1.0 / -0.0
try:
    _ = _one_f / _negzero_f
    raise AssertionError("1.0 / -0.0 must raise ZeroDivisionError")
except ZeroDivisionError:
    _ledger.append(1)

# 1.0 % 0.0
try:
    _ = _one_f % _zero_f
    raise AssertionError("1.0 % 0.0 must raise ZeroDivisionError")
except ZeroDivisionError:
    _ledger.append(1)

# divmod(1.0, 0.0)
try:
    _ = divmod(_one_f, _zero_f)
    raise AssertionError("divmod(1.0, 0.0) must raise ZeroDivisionError")
except ZeroDivisionError:
    _ledger.append(1)

# divmod(0.0, 0.0)
try:
    _ = divmod(_zero_f, _zero_f)
    raise AssertionError("divmod(0.0, 0.0) must raise ZeroDivisionError")
except ZeroDivisionError:
    _ledger.append(1)

# divmod(2.5, 0.0)
try:
    _ = divmod(_twohalf_f, _zero_f)
    raise AssertionError("divmod(2.5, 0.0) must raise ZeroDivisionError")
except ZeroDivisionError:
    _ledger.append(1)

# divmod(-1.5, 0.0)
try:
    _ = divmod(_neg_oneandhalf_f, _zero_f)
    raise AssertionError("divmod(-1.5, 0.0) must raise ZeroDivisionError")
except ZeroDivisionError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_float_divzero_silent_inf {sum(_ledger)} asserts")
