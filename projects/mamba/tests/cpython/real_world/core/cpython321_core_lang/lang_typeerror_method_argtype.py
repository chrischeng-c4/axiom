# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_typeerror_method_argtype"
# subject = "cpython321.lang_typeerror_method_argtype"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_typeerror_method_argtype.py"
# status = "filled"
# ///
"""cpython321.lang_typeerror_method_argtype: execute CPython 3.12 seed lang_typeerror_method_argtype"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Spec seed for CPython TypeError contract on wrong-type arguments
# to built-in methods/functions. Surface: CPython raises TypeError
# when a method or builtin receives an argument of a type it can't
# handle, even though call-shape is correct:
#   • str.join over non-str iterable (int, None, bytes);
#   • bytes.join over non-bytes iterable (int, str);
#   • str.replace / .find / .startswith / .split / .count with int
#     where a str is required;
#   • `1 in "abc"` and `"a" in b"abc"` — mixed text/bytes/int probes;
#   • str.center with int fillchar (must be a one-char str);
#   • round() / abs() on a str or list;
#   • divmod / pow on str — neither type supports __divmod__ / __pow__
#     with itself.
#
# Mamba 0.3.60 currently DOES NOT raise TypeError on any of these
# forms; each invocation silently returns a wrong-shape value (None,
# the original receiver, or 0). This seed pins Fail today so the
# runner surfaces drift when mamba grows arg-type rejection on built-
# in methods.
#
# `Any`-typed holders push the probe past static checkers (Pyright)
# and mamba's compile-time argtype enforcement so the runtime
# divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_s: Any = "abc"
_b: Any = b"abc"
_i: Any = 1
_ints: Any = [1, 2]
_mixed_none: Any = [None, "a"]
_mixed_int: Any = ("a", 1)
_strs: Any = ["a", "b"]

# str.join over [int, int]
try:
    _ = "".join(_ints)
    raise AssertionError("''.join([1,2]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# str.join over [None, str]
try:
    _ = "".join(_mixed_none)
    raise AssertionError("''.join([None,'a']) must raise TypeError")
except TypeError:
    _ledger.append(1)

# str.join over (str, int) tuple
try:
    _ = "".join(_mixed_int)
    raise AssertionError("''.join(('a',1)) must raise TypeError")
except TypeError:
    _ledger.append(1)

# str.join over bytes — iterates as ints
try:
    _ = "".join(_b)
    raise AssertionError("''.join(b'abc') must raise TypeError")
except TypeError:
    _ledger.append(1)

# bytes.join over [int, int]
try:
    _ = b"".join(_ints)
    raise AssertionError("b''.join([1,2]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# bytes.join over [str, str] — str/bytes boundary
try:
    _ = b"".join(_strs)
    raise AssertionError("b''.join(['a','b']) must raise TypeError")
except TypeError:
    _ledger.append(1)

# str.replace(int, str)
try:
    _ = _s.replace(1, "x")
    raise AssertionError("'abc'.replace(1, 'x') must raise TypeError")
except TypeError:
    _ledger.append(1)

# str.find(int)
try:
    _ = _s.find(1)
    raise AssertionError("'abc'.find(1) must raise TypeError")
except TypeError:
    _ledger.append(1)

# str.startswith(int)
try:
    _ = _s.startswith(1)
    raise AssertionError("'abc'.startswith(1) must raise TypeError")
except TypeError:
    _ledger.append(1)

# str.split(int)
try:
    _ = _s.split(1)
    raise AssertionError("'abc'.split(1) must raise TypeError")
except TypeError:
    _ledger.append(1)

# str.count(int)
try:
    _ = _s.count(1)
    raise AssertionError("'abc'.count(1) must raise TypeError")
except TypeError:
    _ledger.append(1)

# 1 in "abc" — int into str membership
try:
    _ = _i in _s
    raise AssertionError("1 in 'abc' must raise TypeError")
except TypeError:
    _ledger.append(1)

# "a" in b"abc" — str into bytes membership
try:
    _ = "a" in _b
    raise AssertionError("'a' in b'abc' must raise TypeError")
except TypeError:
    _ledger.append(1)

# str.center(5, 1) — fillchar must be str
_a: Any = "a"
_fill_int: Any = 1
try:
    _ = _a.center(5, _fill_int)
    raise AssertionError("'a'.center(5, 1) must raise TypeError")
except TypeError:
    _ledger.append(1)

# round("abc") — round of str
_round: Any = round
try:
    _ = _round(_s)
    raise AssertionError("round('abc') must raise TypeError")
except TypeError:
    _ledger.append(1)

# abs("abc") — abs of str
_abs: Any = abs
try:
    _ = _abs(_s)
    raise AssertionError("abs('abc') must raise TypeError")
except TypeError:
    _ledger.append(1)

# abs([1, 2]) — abs of list
_lst: Any = [1, 2]
try:
    _ = _abs(_lst)
    raise AssertionError("abs([1,2]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# divmod("a", "b")
_divmod: Any = divmod
_sb: Any = "b"
try:
    _ = _divmod(_s, _sb)
    raise AssertionError("divmod('a','b') must raise TypeError")
except TypeError:
    _ledger.append(1)

# pow("a", 2)
_pow: Any = pow
try:
    _ = _pow(_s, 2)
    raise AssertionError("pow('a',2) must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_typeerror_method_argtype {sum(_ledger)} asserts")
