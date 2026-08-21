# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_overflowerror_math_silent"
# subject = "cpython321.lang_overflowerror_math_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_overflowerror_math_silent.py"
# status = "filled"
# ///
"""cpython321.lang_overflowerror_math_silent: execute CPython 3.12 seed lang_overflowerror_math_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Spec seed for CPython OverflowError contract on math-domain
# overflows and on float() of an integer too large to fit the IEEE-754
# double-precision range. Surface: CPython raises
#   OverflowError("math range error")
# on transcendental functions whose result exceeds DBL_MAX, and
#   OverflowError("int too large to convert to float")
# on float(huge_int) when the integer's magnitude exceeds the float
# representable range. mamba 0.3.60 silently returns `inf` (or `0.0`
# in the float()-of-huge-int direction) on every one of these forms.
#
# Probes:
#   • math.exp(1000) — exp grows past DBL_MAX around exp(709.78);
#   • math.exp(710) — just past the float overflow boundary;
#   • math.sinh(1000) — sinh has the same exp-style overflow;
#   • math.cosh(1000) — cosh likewise;
#   • math.pow(2, 10000) — 2 ** 10000 ≫ DBL_MAX (~1.8e308);
#   • math.pow(10, 400) — 10 ** 400 ≫ DBL_MAX;
#   • math.expm1(1000) — expm1 ≈ exp − 1, same overflow band;
#   • float(10 ** 500) — int-to-float on an int whose log10 ≫ 308;
#   • float(-(10 ** 500)) — negative-side same overflow;
#   • 2.0 ** 10000 — `float ** int` follows the math.pow contract.
#
# CPython contract:
#   math.exp(x_big)    → OverflowError("math range error");
#   math.sinh(x_big)   → OverflowError("math range error");
#   math.cosh(x_big)   → OverflowError("math range error");
#   math.pow(b, e_big) → OverflowError("math range error");
#   math.expm1(x_big)  → OverflowError("math range error");
#   float(huge_int)    → OverflowError("int too large to convert to float");
#   float ** big_int   → OverflowError on overflow (libm errno=ERANGE).
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so the
# runtime divergence is what's exercised.
import math
from typing import Any
_ledger: list[int] = []

# math.exp(1000) — far past exp() overflow boundary (~709.78)
try:
    _ = math.exp(1000)
    raise AssertionError("math.exp(1000) must raise OverflowError")
except OverflowError:
    _ledger.append(1)

# math.exp(710) — just past the overflow boundary
try:
    _ = math.exp(710)
    raise AssertionError("math.exp(710) must raise OverflowError")
except OverflowError:
    _ledger.append(1)

# math.sinh(1000) — sinh inherits exp's overflow ceiling
try:
    _ = math.sinh(1000)
    raise AssertionError("math.sinh(1000) must raise OverflowError")
except OverflowError:
    _ledger.append(1)

# math.cosh(1000) — cosh inherits exp's overflow ceiling
try:
    _ = math.cosh(1000)
    raise AssertionError("math.cosh(1000) must raise OverflowError")
except OverflowError:
    _ledger.append(1)

# math.pow(2, 10000) — 2 ** 10000 ≫ DBL_MAX
try:
    _ = math.pow(2, 10000)
    raise AssertionError("math.pow(2, 10000) must raise OverflowError")
except OverflowError:
    _ledger.append(1)

# math.pow(10, 400) — 10 ** 400 ≫ DBL_MAX
try:
    _ = math.pow(10, 400)
    raise AssertionError("math.pow(10, 400) must raise OverflowError")
except OverflowError:
    _ledger.append(1)

# math.expm1(1000) — expm1 ≈ exp − 1, same overflow band
try:
    _ = math.expm1(1000)
    raise AssertionError("math.expm1(1000) must raise OverflowError")
except OverflowError:
    _ledger.append(1)

# float(10 ** 500) — huge int that exceeds DBL_MAX (~1.8e308)
try:
    _ = float(10 ** 500)
    raise AssertionError("float(10 ** 500) must raise OverflowError")
except OverflowError:
    _ledger.append(1)

# float(-(10 ** 500)) — negative-side overflow
try:
    _ = float(-(10 ** 500))
    raise AssertionError("float(-(10 ** 500)) must raise OverflowError")
except OverflowError:
    _ledger.append(1)

# 2.0 ** 10000 — `float ** int` overflow via libm errno=ERANGE
_two: Any = 2.0
try:
    _ = _two ** 10000
    raise AssertionError("2.0 ** 10000 must raise OverflowError")
except OverflowError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_overflowerror_math_silent {sum(_ledger)} asserts")
