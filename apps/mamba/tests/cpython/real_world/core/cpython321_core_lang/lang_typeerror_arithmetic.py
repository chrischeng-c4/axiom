# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_typeerror_arithmetic"
# subject = "cpython321.lang_typeerror_arithmetic"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_typeerror_arithmetic.py"
# status = "filled"
# ///
"""cpython321.lang_typeerror_arithmetic: execute CPython 3.12 seed lang_typeerror_arithmetic"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Spec seed for cross-type arithmetic TypeError (CPython contract).
# Surface: CPython's strong dynamic typing rejects arithmetic between
# unrelated types — int+str, str-str, list-list, dict|set, etc. —
# raising TypeError at evaluation time. Mamba currently DOES NOT
# raise TypeError on these forms (silently returns None instead of
# dispatching __add__/__sub__/__mul__/.../TypeError fallback). This
# seed encodes the CPython contract; promotion to pass/ tracks when
# mamba grows operator-dispatch strictness.
# Probe (2026-05-26): mamba 0.3.60 — 0/22 forms raise TypeError;
# every operation returns None.
#
# Each try-block expects a TypeError; on a silent NO-RAISE the
# `raise AssertionError(...)` fires, which the runner classifies as
# Fail — matching the `spec/` contract today.
#
# `Any`-typed holders keep static type-checkers (Pyright) from
# flagging the intentional operator mismatches before runtime.
from typing import Any
_ledger: list[int] = []

_i: Any = 1
_s: Any = "a"
_lst: Any = [1, 2, 3]
_lst2: Any = [1]
_tup: Any = (3, 4)
_dct: Any = {"a": 1}
_n: Any = None
_b: Any = b"b"
_set: Any = {1, 2}

# int + str → TypeError ("unsupported operand type(s) for +: 'int' and 'str'")
try:
    _ = _i + _s
    raise AssertionError("int + str must raise TypeError")
except TypeError:
    _ledger.append(1)

# str - str → TypeError (str has no __sub__)
try:
    _ = _s - _s
    raise AssertionError("str - str must raise TypeError")
except TypeError:
    _ledger.append(1)

# list - list → TypeError (list has no __sub__)
try:
    _ = _lst - _lst2
    raise AssertionError("list - list must raise TypeError")
except TypeError:
    _ledger.append(1)

# list + tuple → TypeError (concatenation requires same sequence type)
try:
    _ = _lst + _tup
    raise AssertionError("list + tuple must raise TypeError")
except TypeError:
    _ledger.append(1)

# tuple + list → TypeError (reverse direction)
try:
    _ = _tup + _lst
    raise AssertionError("tuple + list must raise TypeError")
except TypeError:
    _ledger.append(1)

# str * str → TypeError (str.__mul__ requires int)
try:
    _ = _s * _s
    raise AssertionError("str * str must raise TypeError")
except TypeError:
    _ledger.append(1)

# list * list → TypeError (list.__mul__ requires int)
try:
    _ = _lst * _lst2
    raise AssertionError("list * list must raise TypeError")
except TypeError:
    _ledger.append(1)

# int / str → TypeError
try:
    _ = _i / _s
    raise AssertionError("int / str must raise TypeError")
except TypeError:
    _ledger.append(1)

# int + list → TypeError
try:
    _ = _i + _lst
    raise AssertionError("int + list must raise TypeError")
except TypeError:
    _ledger.append(1)

# int + dict → TypeError
try:
    _ = _i + _dct
    raise AssertionError("int + dict must raise TypeError")
except TypeError:
    _ledger.append(1)

# None + int → TypeError
try:
    _ = _n + _i
    raise AssertionError("None + int must raise TypeError")
except TypeError:
    _ledger.append(1)

# int + None → TypeError (reverse)
try:
    _ = _i + _n
    raise AssertionError("int + None must raise TypeError")
except TypeError:
    _ledger.append(1)

# str + bytes → TypeError (text/bytes boundary, py3 strict)
try:
    _ = _s + _b
    raise AssertionError("str + bytes must raise TypeError")
except TypeError:
    _ledger.append(1)

# bytes + str → TypeError (reverse boundary)
try:
    _ = _b + _s
    raise AssertionError("bytes + str must raise TypeError")
except TypeError:
    _ledger.append(1)

# int << float → TypeError (shift requires int)
_f: Any = 2.0
try:
    _ = _i << _f
    raise AssertionError("int << float must raise TypeError")
except TypeError:
    _ledger.append(1)

# int & str → TypeError (bitwise requires same int family)
try:
    _ = _i & _s
    raise AssertionError("int & str must raise TypeError")
except TypeError:
    _ledger.append(1)

# set | list → TypeError (set union requires set/frozenset)
try:
    _ = _set | _lst
    raise AssertionError("set | list must raise TypeError")
except TypeError:
    _ledger.append(1)

# dict | list → TypeError (PEP 584 dict union requires dict)
try:
    _ = _dct | _lst
    raise AssertionError("dict | list must raise TypeError")
except TypeError:
    _ledger.append(1)

# unary minus on str → TypeError
try:
    _ = -_s
    raise AssertionError("unary minus on str must raise TypeError")
except TypeError:
    _ledger.append(1)

# unary minus on list → TypeError
try:
    _ = -_lst
    raise AssertionError("unary minus on list must raise TypeError")
except TypeError:
    _ledger.append(1)

# unary ~ on str → TypeError (bitwise complement requires int)
try:
    _ = ~_s
    raise AssertionError("unary ~ on str must raise TypeError")
except TypeError:
    _ledger.append(1)

# int % str → TypeError on int side (note: "fmt" % args is str.__mod__,
# different operand; here int 10 has __mod__ that rejects str)
try:
    _ = _i % _s
    raise AssertionError("int % str must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_typeerror_arithmetic {sum(_ledger)} asserts")
