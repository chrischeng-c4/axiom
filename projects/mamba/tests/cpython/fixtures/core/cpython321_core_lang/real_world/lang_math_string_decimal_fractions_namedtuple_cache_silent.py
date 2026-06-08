# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_math_string_decimal_fractions_namedtuple_cache_silent"
# subject = "cpython321.lang_math_string_decimal_fractions_namedtuple_cache_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_math_string_decimal_fractions_namedtuple_cache_silent.py"
# status = "filled"
# ///
"""cpython321.lang_math_string_decimal_fractions_namedtuple_cache_silent: execute CPython 3.12 seed lang_math_string_decimal_fractions_namedtuple_cache_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the
# silent value-contract divergence of the
# `math` / `string` / `decimal` / `fractions` /
# `collections.namedtuple` / `functools.cache` six-pack
# pinned to atomic 231:
# `math` (the documented extended `hasattr(math, "erf") /
# "erfc" / "gamma" / "lgamma" == True` special-function
# surface), `string.Template` / `string.Formatter` (the
# documented `Template("Hello, $name").substitute(name="world")
# == "Hello, world"` and `Formatter().format("Hello, {0}",
# "world") == "Hello, world"` instance value contracts —
# mamba's `string.Template(...)` / `string.Formatter()`
# return dict handles so `.substitute(...)` / `.format(...)`
# raise AttributeError at call site), `decimal` (the
# documented `Decimal("1.1") + Decimal("2.2") ==
# Decimal("3.3")` arithmetic value contract — mamba silently
# returns the boxed-handle integer `-140737488355327` and
# `Decimal("1.5") == Decimal("1.50")` collapses to False),
# `fractions` (the documented `Fraction(1, 3)` str-form value
# contract — mamba silently renders as the boxed-handle
# integer `1099511627776`, and `Fraction(2, 4) ==
# Fraction(1, 2)` collapses to False because the two boxed
# handles do not match), `collections.namedtuple` (the
# documented `P(1, 2).x == 1` accessor value contract —
# mamba's `P(1, 2).x` silently returns None), and
# `functools.cache` / `functools.lru_cache` (the documented
# recursive-decorated-fn value contract — mamba's `@cache`
# decorated recursive `fib(10)` returns None instead of 55).
#
# Behavioral edges that CONFORM on mamba (cmath full surface
# + value ops, math number-theory + arithmetic + trig
# identities, itertools finite-output value ops, operator
# value ops, collections Counter / OrderedDict / defaultdict
# / deque / ChainMap value ops, functools.reduce + partial
# value ops, string.capwords value op) are covered in the
# matching pass fixture
# `test_cmath_math_itertools_operator_collections_value_ops`.
from typing import Any
import math as _math_mod
import string as _string_mod
import decimal as _decimal_mod
import fractions as _fractions_mod
import collections as _collections_mod
import functools as _functools_mod

math: Any = _math_mod
string: Any = _string_mod
decimal: Any = _decimal_mod
fractions: Any = _fractions_mod
collections: Any = _collections_mod
functools: Any = _functools_mod


_ledger: list[int] = []

# 1) math — special-function surface hasattr
#    (mamba: erf / erfc / gamma / lgamma all False)
assert hasattr(math, "erf") == True; _ledger.append(1)
assert hasattr(math, "erfc") == True; _ledger.append(1)
assert hasattr(math, "gamma") == True; _ledger.append(1)
assert hasattr(math, "lgamma") == True; _ledger.append(1)

# 2) string.Template — instance substitution value contract
#    (mamba: substitute / safe_substitute raise AttributeError
#    at call site because `Template(...)` returns a dict)
try:
    _t = string.Template("Hello, $name")
    _r = _t.substitute(name="world")
    _ok = _r == "Hello, world"
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)

# 3) string.Formatter — instance format value contract
#    (mamba: format raises AttributeError at call site
#    because `Formatter()` returns a dict)
try:
    _f = string.Formatter()
    _r = _f.format("Hello, {0}", "world")
    _ok = _r == "Hello, world"
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)

# 4) decimal.Decimal — arithmetic value contract
#    (mamba: + silently returns the boxed-handle integer
#    -140737488355327 instead of Decimal('3.3'))
assert decimal.Decimal("1.1") + decimal.Decimal("2.2") == decimal.Decimal("3.3"); _ledger.append(1)

# 5) decimal.Decimal — trailing-zero equality contract
#    (mamba: Decimal('1.5') == Decimal('1.50') collapses to
#    False because the boxed handles differ)
assert (decimal.Decimal("1.5") == decimal.Decimal("1.50")) == True; _ledger.append(1)

# 6) fractions.Fraction — str-form value contract
#    (mamba: str(Fraction(1, 3)) silently renders as the
#    boxed-handle integer '1099511627776' instead of '1/3')
assert str(fractions.Fraction(1, 3)) == "1/3"; _ledger.append(1)

# 7) fractions.Fraction — reduce-equivalence value contract
#    (mamba: Fraction(2, 4) == Fraction(1, 2) collapses to
#    False because the two boxed handles do not match)
assert (fractions.Fraction(2, 4) == fractions.Fraction(1, 2)) == True; _ledger.append(1)

# 8) collections.namedtuple — accessor value contract
#    (mamba: P(1, 2).x silently returns None)
_P = collections.namedtuple("P", "x y")
_p = _P(1, 2)
assert _p.x == 1; _ledger.append(1)
assert _p.y == 2; _ledger.append(1)

# 9) functools.cache — recursive-decorated-fn value contract
#    (mamba: @cache-decorated recursive fib(10) silently
#    returns None instead of 55)
@functools.cache
def _fib(n: int) -> int:
    return n if n < 2 else _fib(n - 1) + _fib(n - 2)


assert _fib(10) == 55; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_math_string_decimal_fractions_namedtuple_cache_silent {sum(_ledger)} asserts")
