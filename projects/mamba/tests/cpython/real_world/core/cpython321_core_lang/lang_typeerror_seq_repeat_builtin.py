# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_typeerror_seq_repeat_builtin"
# subject = "cpython321.lang_typeerror_seq_repeat_builtin"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_typeerror_seq_repeat_builtin.py"
# status = "filled"
# ///
"""cpython321.lang_typeerror_seq_repeat_builtin: execute CPython 3.12 seed lang_typeerror_seq_repeat_builtin"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Spec seed for CPython TypeError contract on sequence repetition with
# a non-int multiplier, `range()` with a non-int bound, `abs()` /
# `bin()` / `hex()` / `oct()` against non-numeric operands, `%`
# string-format mismatch, and cross-type augmented assignment. Surface:
# CPython rejects every form below with TypeError; mamba 0.3.60
# silently returns `None`, an empty range, `0`, the wrong formatted
# string, or leaves the augmented-assignment target as `None`.
# Existing lang_typeerror_* seeds cover binary arithmetic / call-arity
# / unhashable / iter-required / immutable-mutation / ordering /
# numeric-conversion-constructor / bitwise-unary; this seed adds the
# seq-repeat-by-float, range-with-float, abs-of-non-numeric,
# bin/hex/oct-of-non-int, %-format-mismatch, and cross-type
# augmented-assignment angles.
#
# Probes (every form CPython raises TypeError on, mamba silently
# returns a wrong-shape value):
#   • 'abc' * 1.5       → mamba: None
#   • [1] * 1.5         → mamba: None
#   • (1,) * 1.5        → mamba: None
#   • range(1.5)        → mamba: empty range (list() == [])
#   • abs('abc')        → mamba: 0
#   • abs([])           → mamba: 0
#   • abs(None)         → mamba: 0
#   • bin(1.5)          → mamba: None
#   • hex(1.5)          → mamba: None
#   • oct(1.5)          → mamba: None
#   • bin('abc')        → mamba: None
#   • bin(None)         → mamba: None
#   • '%d' % 'abc'      → mamba: '0' (silently coerces to 0)
#   • '%d' % []         → mamba: '0'
#   • _x = 1; _x += 'a' → mamba: _x becomes None
#   • _x = 'a'; _x += 1 → mamba: _x becomes None
#
# CPython contract:
#   seq * non_int     → TypeError("can't multiply sequence by non-int
#                          of type '<typename>'");
#   range(non_int)    → TypeError("'<typename>' object cannot be
#                          interpreted as an integer");
#   abs(non_numeric)  → TypeError("bad operand type for abs():
#                          '<typename>'");
#   bin(non_int)      → TypeError("'<typename>' object cannot be
#                          interpreted as an integer");
#   '%d' % non_numeric → TypeError("%d format: a real number is
#                          required, not <typename>");
#   int += str        → TypeError("unsupported operand type(s) for +=:
#                          'int' and 'str'") (augmented += dispatches
#                          to __iadd__ then __add__ on int, same
#                          binary path).
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so the
# runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_str_a: Any = "abc"
_lst: Any = [1, 2]
_tup: Any = (1, 2)
_flt: Any = 1.5
_None: Any = None

# 'abc' * 1.5 — string repetition by float
try:
    _ = _str_a * _flt
    raise AssertionError("'abc' * 1.5 must raise TypeError")
except TypeError:
    _ledger.append(1)

# [1, 2] * 1.5 — list repetition by float
try:
    _ = _lst * _flt
    raise AssertionError("[1, 2] * 1.5 must raise TypeError")
except TypeError:
    _ledger.append(1)

# (1, 2) * 1.5 — tuple repetition by float
try:
    _ = _tup * _flt
    raise AssertionError("(1, 2) * 1.5 must raise TypeError")
except TypeError:
    _ledger.append(1)

# range(1.5) — range with float bound
try:
    _ = list(range(_flt))
    raise AssertionError("range(1.5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# abs('abc') — abs of str
try:
    _ = abs(_str_a)
    raise AssertionError("abs('abc') must raise TypeError")
except TypeError:
    _ledger.append(1)

# abs([1, 2]) — abs of list
try:
    _ = abs(_lst)
    raise AssertionError("abs([1, 2]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# abs(None) — abs of NoneType
try:
    _ = abs(_None)
    raise AssertionError("abs(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# bin(1.5) — bin requires int, not float
try:
    _ = bin(_flt)
    raise AssertionError("bin(1.5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# hex(1.5) — hex requires int, not float
try:
    _ = hex(_flt)
    raise AssertionError("hex(1.5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# oct(1.5) — oct requires int, not float
try:
    _ = oct(_flt)
    raise AssertionError("oct(1.5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# bin('abc') — bin requires int, not str
try:
    _ = bin(_str_a)
    raise AssertionError("bin('abc') must raise TypeError")
except TypeError:
    _ledger.append(1)

# bin(None) — bin requires int, not NoneType
try:
    _ = bin(_None)
    raise AssertionError("bin(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# '%d' % 'abc' — %d requires a real number
_pct_fmt: Any = "%d"
try:
    _ = _pct_fmt % _str_a
    raise AssertionError("'%d' % 'abc' must raise TypeError")
except TypeError:
    _ledger.append(1)

# '%d' % [] — %d requires a real number, not a list
try:
    _ = _pct_fmt % _lst
    raise AssertionError("'%d' % [1, 2] must raise TypeError")
except TypeError:
    _ledger.append(1)

# int += str — augmented assignment cross-type
try:
    _aug: Any = 1
    _aug += "a"
    raise AssertionError("int += str must raise TypeError")
except TypeError:
    _ledger.append(1)

# str += int — augmented assignment cross-type
try:
    _aug2: Any = "a"
    _aug2 += 1
    raise AssertionError("str += int must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_typeerror_seq_repeat_builtin {sum(_ledger)} asserts")
