# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_typeerror_builtin_silent_coerce"
# subject = "cpython321.lang_typeerror_builtin_silent_coerce"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_typeerror_builtin_silent_coerce.py"
# status = "filled"
# ///
"""cpython321.lang_typeerror_builtin_silent_coerce: execute CPython 3.12 seed lang_typeerror_builtin_silent_coerce"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Spec seed for CPython TypeError / ValueError contract on the
# top-level builtin functions that mamba silently coerces to a
# zero/empty value instead of dispatching the type-error fallback.
# Surface: CPython rejects (1) `sum(['a','b'])` and `sum([[1],[2]])`
# because `int + str/list` is undefined (start defaults to int 0), (2)
# `''.join([1,2])` and `b''.join(['a','b'])` because the join argument
# must be a homogeneous sequence of the host type, (3) `min([])` /
# `max([])` because there's no neutral element (ValueError, not
# TypeError, on empty without `default=`), (4) `min(['a', 1])` because
# `str < int` is undefined, (5) `abs(str)` / `round(str)` / `pow(str,
# str)` because no `__abs__` / `__round__` / `__pow__` protocol is
# defined for str.
#
# Probes (every form CPython rejects, mamba silently coerces):
#   • sum(['a','b','c'])             → mamba: 0      (TypeError)
#   • sum([[1],[2]])                 → mamba: 0      (TypeError)
#   • ''.join([1, 2, 3])             → mamba: ''     (TypeError)
#   • b''.join(['a', 'b'])           → mamba: b''    (TypeError)
#   • min([])                        → mamba: None   (ValueError)
#   • max([])                        → mamba: None   (ValueError)
#   • min(['a', 1])                  → mamba: 1      (TypeError)
#   • max([1, 'a'])                  → mamba: depends (TypeError)
#   • abs('hello')                   → mamba: 0      (TypeError)
#   • round('hello')                 → mamba: 0      (TypeError)
#   • pow('a', 'b')                  → mamba: None   (TypeError)
#
# CPython contract:
#   sum([str, ...])           → TypeError("unsupported operand type(s)
#                                     for +: 'int' and 'str'");
#   sum([list, ...])          → TypeError("unsupported operand type(s)
#                                     for +: 'int' and 'list'");
#   str.join([int, ...])      → TypeError("sequence item 0: expected
#                                     str instance, int found");
#   bytes.join([str, ...])    → TypeError("sequence item 0: expected
#                                     a bytes-like object, str found");
#   min(empty_seq)            → ValueError("min() iterable argument is
#                                     empty");
#   max(empty_seq)            → ValueError("max() iterable argument is
#                                     empty");
#   min(['a', 1])             → TypeError("'<' not supported between
#                                     instances of 'int' and 'str'");
#   abs(str)                  → TypeError("bad operand type for
#                                     abs(): 'str'");
#   round(str)                → TypeError("type str doesn't define
#                                     __round__ method");
#   pow(str, str)             → TypeError("unsupported operand type(s)
#                                     for ** or pow(): 'str' and
#                                     'str'").
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so the
# runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_strs: Any = ["a", "b", "c"]
_lists: Any = [[1], [2]]
_ints_to_join: Any = [1, 2, 3]
_strs_to_bjoin: Any = ["a", "b"]
_empty: Any = []
_mixed: Any = ["a", 1]
_mixed_rev: Any = [1, "a"]
_s: Any = "hello"
_s2: Any = "a"
_s3: Any = "b"

# sum(['a','b','c']) — start=0 (int) cannot add str
try:
    _ = sum(_strs)
    raise AssertionError("sum([str,...]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# sum([[1],[2]]) — start=0 (int) cannot add list
try:
    _ = sum(_lists)
    raise AssertionError("sum([list,...]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# ''.join([1,2,3]) — join requires str elements
try:
    _ = "".join(_ints_to_join)
    raise AssertionError("str.join([int,...]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# b''.join(['a','b']) — bytes.join requires bytes-like elements
try:
    _ = b"".join(_strs_to_bjoin)
    raise AssertionError("bytes.join([str,...]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# min([]) — empty iterable is ValueError
try:
    _ = min(_empty)
    raise AssertionError("min([]) must raise ValueError")
except ValueError:
    _ledger.append(1)

# max([]) — empty iterable is ValueError
try:
    _ = max(_empty)
    raise AssertionError("max([]) must raise ValueError")
except ValueError:
    _ledger.append(1)

# min(['a', 1]) — str/int ordering is undefined
try:
    _ = min(_mixed)
    raise AssertionError("min(['a', 1]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# max([1, 'a']) — same problem in the other direction
try:
    _ = max(_mixed_rev)
    raise AssertionError("max([1, 'a']) must raise TypeError")
except TypeError:
    _ledger.append(1)

# abs('hello') — str has no __abs__
try:
    _ = abs(_s)
    raise AssertionError("abs(str) must raise TypeError")
except TypeError:
    _ledger.append(1)

# round('hello') — str has no __round__
try:
    _ = round(_s)
    raise AssertionError("round(str) must raise TypeError")
except TypeError:
    _ledger.append(1)

# pow('a', 'b') — str has no __pow__
try:
    _ = pow(_s2, _s3)
    raise AssertionError("pow(str, str) must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_typeerror_builtin_silent_coerce {sum(_ledger)} asserts")
