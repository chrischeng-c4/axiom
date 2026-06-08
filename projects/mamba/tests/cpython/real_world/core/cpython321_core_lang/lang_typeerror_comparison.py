# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_typeerror_comparison"
# subject = "cpython321.lang_typeerror_comparison"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_typeerror_comparison.py"
# status = "filled"
# ///
"""cpython321.lang_typeerror_comparison: execute CPython 3.12 seed lang_typeerror_comparison"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Spec seed for CPython TypeError contract on ordering comparisons.
# Surface: CPython raises TypeError on `< <= > >=` between unrelated
# types (int vs str, list vs set, dict vs dict, None vs anything,
# bytes vs str, complex vs complex). Equality (`==`, `!=`) does
# *not* raise — CPython falls back to identity / False — so we
# only probe ordering operators here.
#
# Mamba 0.3.60 currently DOES NOT raise TypeError on any of these
# forms; every `<` `<=` `>` `>=` between mismatched types silently
# returns False. This seed pins Fail today so the runner surfaces
# drift when mamba grows rich-comparison strictness.
#
# `Any`-typed holders keep static type-checkers (Pyright) from
# flagging the intentional cross-type ordering before runtime.
from typing import Any
_ledger: list[int] = []

_i: Any = 1
_s: Any = "a"
_lst: Any = [1]
_set: Any = {1}
_tup: Any = (1,)
_dct: Any = {"a": 1}
_n: Any = None
_by: Any = b"a"

# int < str — primitive vs text
try:
    _ = _i < _s
    raise AssertionError("int < str must raise TypeError")
except TypeError:
    _ledger.append(1)

# str < int — reverse direction
try:
    _ = _s < _i
    raise AssertionError("str < int must raise TypeError")
except TypeError:
    _ledger.append(1)

# list < set — different container kinds
try:
    _ = _lst < _set
    raise AssertionError("list < set must raise TypeError")
except TypeError:
    _ledger.append(1)

# None < None — NoneType has no ordering
try:
    _ = _n < _n
    raise AssertionError("None < None must raise TypeError")
except TypeError:
    _ledger.append(1)

# list < int — container vs primitive
try:
    _ = _lst < _i
    raise AssertionError("list < int must raise TypeError")
except TypeError:
    _ledger.append(1)

# tuple < list — both sequences, but distinct types
try:
    _ = _tup < _lst
    raise AssertionError("tuple < list must raise TypeError")
except TypeError:
    _ledger.append(1)

# dict < dict — mappings have no defined ordering
try:
    _ = _dct < {"b": 2}
    raise AssertionError("dict < dict must raise TypeError")
except TypeError:
    _ledger.append(1)

# set < list — sets order partially but only against set/frozenset
try:
    _ = _set < _lst
    raise AssertionError("set < list must raise TypeError")
except TypeError:
    _ledger.append(1)

# bytes < str — text / bytes boundary
try:
    _ = _by < _s
    raise AssertionError("bytes < str must raise TypeError")
except TypeError:
    _ledger.append(1)

# complex < complex — complex has no defined ordering
_c1: Any = complex(1, 2)
_c2: Any = complex(3, 4)
try:
    _ = _c1 < _c2
    raise AssertionError("complex < complex must raise TypeError")
except TypeError:
    _ledger.append(1)

# int < None
try:
    _ = _i < _n
    raise AssertionError("int < None must raise TypeError")
except TypeError:
    _ledger.append(1)

# None < int (reverse)
try:
    _ = _n < _i
    raise AssertionError("None < int must raise TypeError")
except TypeError:
    _ledger.append(1)

# str > int — `>` operator family
try:
    _ = _s > _i
    raise AssertionError("str > int must raise TypeError")
except TypeError:
    _ledger.append(1)

# list <= tuple — `<=` operator family
try:
    _ = _lst <= _tup
    raise AssertionError("list <= tuple must raise TypeError")
except TypeError:
    _ledger.append(1)

# int >= str — `>=` operator family
try:
    _ = _i >= _s
    raise AssertionError("int >= str must raise TypeError")
except TypeError:
    _ledger.append(1)

# dict <= list — combining mismatched container kinds with `<=`
try:
    _ = _dct <= _lst
    raise AssertionError("dict <= list must raise TypeError")
except TypeError:
    _ledger.append(1)

# float < str — float vs text
_f: Any = 1.5
try:
    _ = _f < _s
    raise AssertionError("float < str must raise TypeError")
except TypeError:
    _ledger.append(1)

# bool < list — bool inherits int but list is not int-comparable
_b: Any = True
try:
    _ = _b < _lst
    raise AssertionError("bool < list must raise TypeError")
except TypeError:
    _ledger.append(1)

# frozenset < list — frozenset has set-ordering only vs sets
_fs: Any = frozenset({1, 2})
try:
    _ = _fs < _lst
    raise AssertionError("frozenset < list must raise TypeError")
except TypeError:
    _ledger.append(1)

# range < list — range has no <
_r: Any = range(3)
try:
    _ = _r < _lst
    raise AssertionError("range < list must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_typeerror_comparison {sum(_ledger)} asserts")
