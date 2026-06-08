# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_typeerror_calls_subscripts"
# subject = "cpython321.lang_typeerror_calls_subscripts"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_typeerror_calls_subscripts.py"
# status = "filled"
# ///
"""cpython321.lang_typeerror_calls_subscripts: execute CPython 3.12 seed lang_typeerror_calls_subscripts"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Spec seed for CPython TypeError contract on calls, subscripts,
# and protocol-required builtins.
# Surface: CPython raises TypeError when invoking a non-callable
# object, subscripting a sequence with a non-integer key,
# subscripting a mapping with an unhashable key, calling
# `len(x)` / `iter(x)` on an object that lacks __len__/__iter__,
# the `in` operator against a non-iterable RHS, and `hash(x)`
# on an object whose type is unhashable (list, dict, set).
# Mamba 0.3.60 currently DOES NOT raise TypeError on any of
# these forms — most return None silently, dict-subscript with
# an unhashable key raises KeyError (not TypeError), and
# hash([])/hash({}) returns a plausible-looking int. This seed
# pins Fail today so the runner surfaces drift when mamba grows
# protocol-error dispatch (mass-promote candidate via
# `git mv spec/lang_typeerror_calls_subscripts.py pass/`).
#
# `Any`-typed holders keep static type-checkers (Pyright) from
# flagging the intentional protocol mismatches before runtime.
from typing import Any
_ledger: list[int] = []

_s: Any = "abc"
_lst: Any = [1, 2, 3]
_tup: Any = (10, 20)
_dct: Any = {"a": 1}
_set: Any = {1, 2}
_i: Any = 42
_n: Any = None
_b: Any = True

# str[str] — non-integer subscript on str
try:
    _ = _s["x"]
    raise AssertionError("str[str] must raise TypeError")
except TypeError:
    _ledger.append(1)

# list[str] — non-integer subscript on list
try:
    _ = _lst["a"]
    raise AssertionError("list[str] must raise TypeError")
except TypeError:
    _ledger.append(1)

# list[float] — float subscript on list
try:
    _ = _lst[1.5]
    raise AssertionError("list[float] must raise TypeError")
except TypeError:
    _ledger.append(1)

# tuple[str] — non-integer subscript on tuple
try:
    _ = _tup["x"]
    raise AssertionError("tuple[str] must raise TypeError")
except TypeError:
    _ledger.append(1)

# set[index] — set has no __getitem__ at all
try:
    _ = _set[0]
    raise AssertionError("set[idx] must raise TypeError")
except TypeError:
    _ledger.append(1)

# dict[list] — unhashable key
try:
    _ = _dct[[1]]
    raise AssertionError("dict[list] must raise TypeError")
except TypeError:
    _ledger.append(1)

# dict[set] — unhashable key (mutable set)
try:
    _ = _dct[{1, 2}]
    raise AssertionError("dict[set] must raise TypeError")
except TypeError:
    _ledger.append(1)

# dict[dict] — unhashable key
try:
    _ = _dct[{"k": 1}]
    raise AssertionError("dict[dict] must raise TypeError")
except TypeError:
    _ledger.append(1)

# len(int) — int has no __len__
try:
    _ = len(_i)
    raise AssertionError("len(int) must raise TypeError")
except TypeError:
    _ledger.append(1)

# len(None) — NoneType has no __len__
try:
    _ = len(_n)
    raise AssertionError("len(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# len(bool) — bool inherits int, no __len__
try:
    _ = len(_b)
    raise AssertionError("len(bool) must raise TypeError")
except TypeError:
    _ledger.append(1)

# iter(int) — int is not iterable
try:
    _ = iter(_i)
    raise AssertionError("iter(int) must raise TypeError")
except TypeError:
    _ledger.append(1)

# iter(None) — NoneType is not iterable
try:
    _ = iter(_n)
    raise AssertionError("iter(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# `in` against non-iterable RHS (int)
try:
    _ = 1 in _i
    raise AssertionError("`1 in int` must raise TypeError")
except TypeError:
    _ledger.append(1)

# `in` against non-iterable RHS (None)
try:
    _ = 1 in _n
    raise AssertionError("`1 in None` must raise TypeError")
except TypeError:
    _ledger.append(1)

# Calling a non-callable (int)
try:
    _ = _i()
    raise AssertionError("int() — calling int instance must raise TypeError")
except TypeError:
    _ledger.append(1)

# Calling a non-callable (None)
try:
    _ = _n()
    raise AssertionError("None() must raise TypeError")
except TypeError:
    _ledger.append(1)

# Calling a non-callable (list)
try:
    _ = _lst()
    raise AssertionError("list() — calling list instance must raise TypeError")
except TypeError:
    _ledger.append(1)

# hash(list) — list is unhashable
try:
    _ = hash([1, 2])
    raise AssertionError("hash(list) must raise TypeError")
except TypeError:
    _ledger.append(1)

# hash(dict) — dict is unhashable
try:
    _ = hash({"a": 1})
    raise AssertionError("hash(dict) must raise TypeError")
except TypeError:
    _ledger.append(1)

# hash(set) — set is unhashable
try:
    _ = hash({1, 2})
    raise AssertionError("hash(set) must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_typeerror_calls_subscripts {sum(_ledger)} asserts")
