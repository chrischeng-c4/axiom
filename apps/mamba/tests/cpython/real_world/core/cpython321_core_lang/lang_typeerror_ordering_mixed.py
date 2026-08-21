# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_typeerror_ordering_mixed"
# subject = "cpython321.lang_typeerror_ordering_mixed"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_typeerror_ordering_mixed.py"
# status = "filled"
# ///
"""cpython321.lang_typeerror_ordering_mixed: execute CPython 3.12 seed lang_typeerror_ordering_mixed"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Spec seed for CPython TypeError contract on cross-type ordering
# comparisons. Surface: Python 3 made `<`, `<=`, `>`, `>=` strict
# about operand types — CPython raises
#   TypeError("'<op>' not supported between instances of '<A>' and
#             '<B>'")
# whenever the two operands are of unrelated types. Mixed-type
# equality (`==` / `!=`) is silently False/True, but ordering is a
# hard error.
#
# Probes:
#   • int < str / str < int / str <= int — numeric vs text;
#   • None < int / int > None — None has no defined ordering;
#   • list < int / list < str / tuple < int / set < int — container
#     vs scalar;
#   • dict < dict — dict is unordered as a type;
#   • str < bytes — text vs bytes are siblings in Python 3 with no
#     mutual ordering;
#   • sorted([1, "a"]) — heterogeneous list sort surfaces the same
#     TypeError because sorted falls back to `<` comparisons.
#
# Mamba 0.3.60 currently DOES NOT raise TypeError on any of these
# forms; the cross-type compare silently returns a boolean (chosen
# by an internal lexicographic-by-type-name ordering rule, the same
# fallback CPython used in Python 2). This seed pins Fail today so
# the runner surfaces drift when mamba grows Python 3-strict
# cross-type ordering rejection.
#
# `Any`-typed holders push the operands past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so the
# runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_s: Any = "abc"
_i: Any = 1
_n: Any = None
_l: Any = [1, 2]
_d: Any = {"a": 1}
_t: Any = (1, 2)
_st: Any = {1, 2}
_b: Any = b"abc"

# int < str
try:
    _ = _i < _s
    raise AssertionError("1 < 'abc' must raise TypeError")
except TypeError:
    _ledger.append(1)

# str < int
try:
    _ = _s < _i
    raise AssertionError("'abc' < 1 must raise TypeError")
except TypeError:
    _ledger.append(1)

# str <= int
try:
    _ = _s <= _i
    raise AssertionError("'abc' <= 1 must raise TypeError")
except TypeError:
    _ledger.append(1)

# None < int
try:
    _ = _n < _i
    raise AssertionError("None < 1 must raise TypeError")
except TypeError:
    _ledger.append(1)

# int > None
try:
    _ = _i > _n
    raise AssertionError("1 > None must raise TypeError")
except TypeError:
    _ledger.append(1)

# list < int
try:
    _ = _l < _i
    raise AssertionError("[1,2] < 1 must raise TypeError")
except TypeError:
    _ledger.append(1)

# list < str
try:
    _ = _l < _s
    raise AssertionError("[1,2] < 'abc' must raise TypeError")
except TypeError:
    _ledger.append(1)

# tuple < int
try:
    _ = _t < _i
    raise AssertionError("(1,2) < 1 must raise TypeError")
except TypeError:
    _ledger.append(1)

# set < int — set's `<` is the strict-subset operator and is only
# defined between sets; with int the comparison raises TypeError
try:
    _ = _st < _i
    raise AssertionError("{1,2} < 1 must raise TypeError")
except TypeError:
    _ledger.append(1)

# dict < dict — dict has no order defined
try:
    _ = _d < _d
    raise AssertionError("dict < dict must raise TypeError")
except TypeError:
    _ledger.append(1)

# str < bytes — text vs bytes have no mutual order in Python 3
try:
    _ = _s < _b
    raise AssertionError("'abc' < b'abc' must raise TypeError")
except TypeError:
    _ledger.append(1)

# sorted on a mixed-type list — falls back to `<` between members
_mixed: Any = [1, "a"]
try:
    _ = sorted(_mixed)
    raise AssertionError("sorted([1, 'a']) must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_typeerror_ordering_mixed {sum(_ledger)} asserts")
