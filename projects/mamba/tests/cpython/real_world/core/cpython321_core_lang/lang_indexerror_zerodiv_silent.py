# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_indexerror_zerodiv_silent"
# subject = "cpython321.lang_indexerror_zerodiv_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_indexerror_zerodiv_silent.py"
# status = "filled"
# ///
"""cpython321.lang_indexerror_zerodiv_silent: execute CPython 3.12 seed lang_indexerror_zerodiv_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Spec seed for CPython IndexError / ZeroDivisionError contracts on
# out-of-range sequence indexing and on dividing by zero. These two
# exception families share a property: CPython raises eagerly at the
# offending operation, mamba 0.3.60 silently returns (None / NaN /
# infinity / a wrong index slot) and lets execution continue.
#
# IndexError surface:
#   • [][0] / [1,2,3][5] — list index past end;
#   • (1,2,3)[5] — tuple index past end;
#   • 'abc'[5] — str index past end;
#   • b'abc'[5] — bytes index past end.
# CPython raises IndexError("list index out of range") and analogous
# messages for tuple/str/bytes. Mamba lets the expression evaluate to
# a sentinel (None or empty) without raising.
#
# ZeroDivisionError surface:
#   • 1.0 / 0.0 — float-by-zero division;
#   • divmod(1, 0) — int divmod by zero;
#   • pow(0, -1) — zero raised to a negative power (CPython:
#     "0.0 cannot be raised to a negative power").
# CPython raises ZeroDivisionError on each. Mamba returns inf / NaN /
# a tuple-of-None / 0 silently.
#
# Mamba 0.3.60 currently does NOT raise on any of these forms; this
# seed pins Fail today so the runner surfaces drift when mamba grows
# CPython-strict bounds-checking and zero-division checks.
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so the
# runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_lst_empty: Any = []
_lst: Any = [1, 2, 3]
_tup: Any = (1, 2, 3)
_s: Any = "abc"
_b: Any = b"abc"
_one_f: Any = 1.0
_zero_f: Any = 0.0
_one: Any = 1
_zero: Any = 0

# [][0] — empty list, any index is out of range
try:
    _ = _lst_empty[0]
    raise AssertionError("[][0] must raise IndexError")
except IndexError:
    _ledger.append(1)

# [1,2,3][5] — index past end of length-3 list
try:
    _ = _lst[5]
    raise AssertionError("[1,2,3][5] must raise IndexError")
except IndexError:
    _ledger.append(1)

# (1,2,3)[5] — tuple index past end
try:
    _ = _tup[5]
    raise AssertionError("(1,2,3)[5] must raise IndexError")
except IndexError:
    _ledger.append(1)

# 'abc'[5] — str index past end
try:
    _ = _s[5]
    raise AssertionError("'abc'[5] must raise IndexError")
except IndexError:
    _ledger.append(1)

# b'abc'[5] — bytes index past end
try:
    _ = _b[5]
    raise AssertionError("b'abc'[5] must raise IndexError")
except IndexError:
    _ledger.append(1)

# 1.0 / 0.0 — float division by zero
try:
    _ = _one_f / _zero_f
    raise AssertionError("1.0 / 0.0 must raise ZeroDivisionError")
except ZeroDivisionError:
    _ledger.append(1)

# divmod(1, 0) — int divmod by zero
try:
    _ = divmod(_one, _zero)
    raise AssertionError("divmod(1, 0) must raise ZeroDivisionError")
except ZeroDivisionError:
    _ledger.append(1)

# pow(0, -1) — zero to a negative power
try:
    _ = pow(_zero, -1)
    raise AssertionError("pow(0, -1) must raise ZeroDivisionError")
except ZeroDivisionError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_indexerror_zerodiv_silent {sum(_ledger)} asserts")
