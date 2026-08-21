# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_typeerror_iter_required"
# subject = "cpython321.lang_typeerror_iter_required"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_typeerror_iter_required.py"
# status = "filled"
# ///
"""cpython321.lang_typeerror_iter_required: execute CPython 3.12 seed lang_typeerror_iter_required"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Spec seed for CPython TypeError contract on iter-required positions.
# Surface: CPython raises TypeError ("'int' object is not iterable",
# "'NoneType' object is not iterable", "object of type 'int' has no
# len()") when a non-iterable value flows into a position that demands
# an iterable / sized:
#   • list / tuple / set / dict / frozenset constructors;
#   • iter() builtin (and next() on non-iterator);
#   • sum / max / min / sorted (when called with single non-iterable);
#   • len() on a type that has no __len__;
#   • for-statement over a non-iterable;
#   • enumerate(non-iter) / zip(non-iter, ...);
#   • *splat / **splat / [*splat] over a non-iterable.
#
# Mamba 0.3.60 currently DOES NOT raise TypeError on most of these
# forms; the non-iterable flows through and the call silently no-ops
# or returns a wrong-shape value. (Two forms — next(1) and next(list)
# — DO raise TypeError on mamba, matching CPython, so they are kept
# out of this seed.)
#
# `Any`-typed holders push the probe past static type-checkers and
# past mamba's compile-time iter-shape check so the runtime divergence
# is what's exercised.
from typing import Any
_ledger: list[int] = []

_i: Any = 1
_n: Any = None
_f: Any = 1.5

# list(non-iterable)
try:
    _ = list(_i)
    raise AssertionError("list(1) must raise TypeError")
except TypeError:
    _ledger.append(1)

# tuple(non-iterable)
try:
    _ = tuple(_i)
    raise AssertionError("tuple(1) must raise TypeError")
except TypeError:
    _ledger.append(1)

# set(non-iterable)
try:
    _ = set(_i)
    raise AssertionError("set(1) must raise TypeError")
except TypeError:
    _ledger.append(1)

# frozenset(non-iterable)
try:
    _ = frozenset(_i)
    raise AssertionError("frozenset(1) must raise TypeError")
except TypeError:
    _ledger.append(1)

# dict(non-iterable, non-mapping)
try:
    _ = dict(_i)
    raise AssertionError("dict(1) must raise TypeError")
except TypeError:
    _ledger.append(1)

# iter(non-iterable)
try:
    _ = iter(_i)
    raise AssertionError("iter(1) must raise TypeError")
except TypeError:
    _ledger.append(1)

# sum(non-iterable)
try:
    _ = sum(_i)
    raise AssertionError("sum(1) must raise TypeError")
except TypeError:
    _ledger.append(1)

# max(non-iterable single arg)
try:
    _ = max(_i)
    raise AssertionError("max(1) must raise TypeError")
except TypeError:
    _ledger.append(1)

# min(non-iterable single arg)
try:
    _ = min(_i)
    raise AssertionError("min(1) must raise TypeError")
except TypeError:
    _ledger.append(1)

# sorted(non-iterable)
try:
    _ = sorted(_i)
    raise AssertionError("sorted(1) must raise TypeError")
except TypeError:
    _ledger.append(1)

# len(non-sized)
try:
    _ = len(_i)
    raise AssertionError("len(1) must raise TypeError")
except TypeError:
    _ledger.append(1)

# len(None)
try:
    _ = len(_n)
    raise AssertionError("len(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# len(float)
try:
    _ = len(_f)
    raise AssertionError("len(1.5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# for x in 1
try:
    for _x in _i:
        pass
    raise AssertionError("for x in 1 must raise TypeError")
except TypeError:
    _ledger.append(1)

# for x in None
try:
    for _x in _n:
        pass
    raise AssertionError("for x in None must raise TypeError")
except TypeError:
    _ledger.append(1)

# list(None)
try:
    _ = list(_n)
    raise AssertionError("list(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# iter(None)
try:
    _ = iter(_n)
    raise AssertionError("iter(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# enumerate(non-iterable)
try:
    _ = list(enumerate(_i))
    raise AssertionError("enumerate(1) must raise TypeError")
except TypeError:
    _ledger.append(1)

# zip(non-iterable, ...)
try:
    _ = list(zip(_i, [1, 2]))
    raise AssertionError("zip(1, ...) must raise TypeError")
except TypeError:
    _ledger.append(1)

# *splat over non-iterable in call
def _f2(*args):
    return args

_f2_any: Any = _f2
try:
    _ = _f2_any(*_i)
    raise AssertionError("f(*1) must raise TypeError")
except TypeError:
    _ledger.append(1)

# **splat over non-mapping in call
try:
    _ = _f2_any(**_i)
    raise AssertionError("f(**1) must raise TypeError")
except TypeError:
    _ledger.append(1)

# [*non-iterable] in list literal splat
try:
    _ = [*_i]
    raise AssertionError("[*1] must raise TypeError")
except TypeError:
    _ledger.append(1)

# {*non-iterable} in set literal splat
try:
    _ = {*_i}
    raise AssertionError("{*1} must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_typeerror_iter_required {sum(_ledger)} asserts")
