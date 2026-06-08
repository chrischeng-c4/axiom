# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_float_int_special_value_silent"
# subject = "cpython321.lang_float_int_special_value_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_float_int_special_value_silent.py"
# status = "filled"
# ///
"""cpython321.lang_float_int_special_value_silent: execute CPython 3.12 seed lang_float_int_special_value_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Spec seed for the CPython numeric-coercion contract around FLOAT
# SPECIAL VALUES (`inf`, `-inf`, `nan`) and WRONG-TYPE / OUT-OF-RANGE
# arguments to `int.from_bytes` / `int.to_bytes`. CPython raises
# `OverflowError`, `ValueError`, or `TypeError` at the BOUNDARY where a
# nonsensical numeric coercion is attempted; mamba either silently
# returns a number (often 0, the original value, or a quasi-bit-pattern)
# or returns `None` — masking the upstream bug where a NaN slipped into
# `int()`, an inf escaped a guard, or a wrong-type argument reached
# `int.from_bytes`.
#
# Surface (CPython rejects, mamba silently coerces):
#   (1) `int(inf)` / `int(-inf)` — OverflowError;
#       `int(nan)` — ValueError;
#   (2) `round(inf)` / `round(-inf)` — OverflowError;
#       `round(nan)` — ValueError;
#   (3) `int.from_bytes(5, 'big')` — TypeError (bytes-like required);
#       `int.from_bytes('abc', 'big')` — TypeError;
#       `int.from_bytes(b'abc', 5)` — TypeError (byteorder must be str);
#       `int.from_bytes(b'abc', 'middle')` — ValueError (only 'big' /
#       'little');
#   (4) `(256).to_bytes(1, 'big')` — OverflowError (too large for 1
#       byte unsigned);
#       `(1).to_bytes(1, 'invalid')` — ValueError (byteorder must be
#       'big' or 'little');
#       `(1).to_bytes(-1, 'big')` — ValueError (length must be
#       non-negative);
#   (5) `int & float` / `int | float` / `int ^ float` / `int << float`
#       / `int >> float` — TypeError (unsupported operand type(s));
#       mamba either returns `None`, `0`, or a quasi-bit-pattern;
#   (6) `(5).numerator` / `(5).denominator` — int's
#       numerator/denominator must be the canonical rational form (n, 1)
#       on CPython; mamba returns `None`;
#   (7) `round(x, ndigits)` — should return the same TYPE as the input
#       and a value mathematically close to `x` rounded to `ndigits`
#       decimals; mamba returns an int bit-pattern instead of a float
#       value when ndigits > 0.
#
# Mamba behavior:
#   • `int(float('inf'))` → 0 (silent)
#   • `int(float('nan'))` → 0 (silent)
#   • `round(float('inf'))` → 0 (silent)
#   • `int.from_bytes(5, 'big')` → 5 (silent — accepts non-bytes)
#   • `int.from_bytes(b'abc', 5)` → some int (silent — accepts non-str)
#   • `int.from_bytes(b'abc', 'middle')` → some int (silent — accepts
#     unknown byteorder)
#   • `(256).to_bytes(1, 'big')` → b'\x00' (silent wrap mod 256)
#   • `(1).to_bytes(1, 'invalid')` → b'\x01' (silent — accepts unknown
#     byteorder, defaults to one form)
#   • `(1).to_bytes(-1, 'big')` → b'' or b'\x01' (silent)
#   • `1 & 1.5` → None (wrong return shape)
#   • `1 << 1.5` → None (wrong return shape)
#   • `(5).numerator` → None (canonical rational form lost)
#   • `(5).denominator` → None
#   • `round(1.25, 1)` → 4608083138725491507 (bit-pattern int — wrong
#     type AND wrong value)
#
# CPython contract:
#   int(inf) / int(-inf) → OverflowError;
#   int(nan) → ValueError;
#   round(inf) / round(-inf) → OverflowError;
#   round(nan) → ValueError;
#   int.from_bytes(non_bytes, 'big') → TypeError;
#   int.from_bytes(bytes, non_str) → TypeError;
#   int.from_bytes(bytes, 'middle') → ValueError;
#   (256).to_bytes(1, 'big') → OverflowError;
#   (1).to_bytes(1, 'invalid') → ValueError;
#   (1).to_bytes(-1, 'big') → ValueError;
#   int & float / | / ^ / << / >> → TypeError;
#   (5).numerator == 5 and (5).denominator == 1;
#   round(1.25, 1) == 1.2 (a float, NOT an int bit-pattern).
#
# `Any`-typed module-level holders push wrong-type operands past
# Pyright + mamba's compile-time argtype enforcement so the runtime
# divergence is exercised.
from typing import Any

_ledger: list[int] = []

_inf: Any = float("inf")
_ninf: Any = float("-inf")
_nan: Any = float("nan")
_int5: Any = 5
_str: Any = "abc"
_f15: Any = 1.5
_i1: Any = 1

# (1) int() on float special values
try:
    _ = int(_inf)
    raise AssertionError("int(inf) must raise OverflowError")
except OverflowError:
    _ledger.append(1)

try:
    _ = int(_ninf)
    raise AssertionError("int(-inf) must raise OverflowError")
except OverflowError:
    _ledger.append(1)

try:
    _ = int(_nan)
    raise AssertionError("int(nan) must raise ValueError")
except ValueError:
    _ledger.append(1)

# (2) round() on float special values
try:
    _ = round(_inf)
    raise AssertionError("round(inf) must raise OverflowError")
except OverflowError:
    _ledger.append(1)

try:
    _ = round(_ninf)
    raise AssertionError("round(-inf) must raise OverflowError")
except OverflowError:
    _ledger.append(1)

try:
    _ = round(_nan)
    raise AssertionError("round(nan) must raise ValueError")
except ValueError:
    _ledger.append(1)

# (3) int.from_bytes wrong types / wrong byteorder values
# int.from_bytes(5, 'big') — TypeError (bytes-like required)
try:
    _ = int.from_bytes(_int5, "big")
    raise AssertionError("int.from_bytes(5, 'big') must raise TypeError")
except TypeError:
    _ledger.append(1)

# int.from_bytes('abc', 'big') — TypeError (str is not bytes-like)
try:
    _ = int.from_bytes(_str, "big")
    raise AssertionError("int.from_bytes('abc', 'big') must raise TypeError")
except TypeError:
    _ledger.append(1)

# int.from_bytes(b'abc', 5) — TypeError (byteorder must be str)
_byteorder_int: Any = 5
try:
    _ = int.from_bytes(b"abc", _byteorder_int)
    raise AssertionError("int.from_bytes(b'abc', 5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# int.from_bytes(b'abc', 'middle') — ValueError (must be 'big' or 'little')
_byteorder_bad: Any = "middle"
try:
    _ = int.from_bytes(b"abc", _byteorder_bad)
    raise AssertionError("int.from_bytes(b'abc', 'middle') must raise ValueError")
except ValueError:
    _ledger.append(1)

# (4) int.to_bytes wrong values
# (256).to_bytes(1, 'big') — OverflowError (256 too large for 1 byte unsigned)
try:
    _ = (256).to_bytes(1, "big")
    raise AssertionError("(256).to_bytes(1, 'big') must raise OverflowError")
except OverflowError:
    _ledger.append(1)

# (1).to_bytes(1, 'invalid') — ValueError (byteorder must be 'big' or 'little')
try:
    _ = (1).to_bytes(1, _byteorder_bad)
    raise AssertionError("(1).to_bytes(1, 'invalid') must raise ValueError")
except ValueError:
    _ledger.append(1)

# (1).to_bytes(-1, 'big') — ValueError (length must be non-negative)
try:
    _ = (1).to_bytes(-1, "big")
    raise AssertionError("(1).to_bytes(-1, 'big') must raise ValueError")
except ValueError:
    _ledger.append(1)

# (65536).to_bytes(2, 'big') — OverflowError
try:
    _ = (65536).to_bytes(2, "big")
    raise AssertionError("(65536).to_bytes(2, 'big') must raise OverflowError")
except OverflowError:
    _ledger.append(1)

# (5) Bitwise operations with float operand
# 1 & 1.5 — TypeError
try:
    _ = _i1 & _f15
    raise AssertionError("1 & 1.5 must raise TypeError")
except TypeError:
    _ledger.append(1)

# 1 | 1.5 — TypeError
try:
    _ = _i1 | _f15
    raise AssertionError("1 | 1.5 must raise TypeError")
except TypeError:
    _ledger.append(1)

# 1 ^ 1.5 — TypeError
try:
    _ = _i1 ^ _f15
    raise AssertionError("1 ^ 1.5 must raise TypeError")
except TypeError:
    _ledger.append(1)

# 1 << 1.5 — TypeError
try:
    _ = _i1 << _f15
    raise AssertionError("1 << 1.5 must raise TypeError")
except TypeError:
    _ledger.append(1)

# 1 >> 1.5 — TypeError
try:
    _ = _i1 >> _f15
    raise AssertionError("1 >> 1.5 must raise TypeError")
except TypeError:
    _ledger.append(1)

# 1.5 & 1 — TypeError (float on left)
try:
    _ = _f15 & _i1
    raise AssertionError("1.5 & 1 must raise TypeError")
except TypeError:
    _ledger.append(1)

# (6) int.numerator / int.denominator — canonical rational form
# CPython: (5).numerator == 5, (5).denominator == 1, (-5).numerator == -5
assert (5).numerator == 5, "(5).numerator must be 5 (not None)"
_ledger.append(1)
assert (5).denominator == 1, "(5).denominator must be 1 (not None)"
_ledger.append(1)
assert (-5).numerator == -5, "(-5).numerator must be -5 (not None)"
_ledger.append(1)
assert (-5).denominator == 1, "(-5).denominator must be 1 (not None)"
_ledger.append(1)
assert (0).numerator == 0, "(0).numerator must be 0 (not None)"
_ledger.append(1)
assert (0).denominator == 1, "(0).denominator must be 1 (not None)"
_ledger.append(1)

# (7) round(x, ndigits) — returns a float for float input, value within
# tolerance. Mamba returns an int bit-pattern (e.g. 4608083138725491507)
# instead of 1.2 — both wrong type AND wrong value.
_r = round(1.25, 1)
assert isinstance(_r, float), f"round(1.25, 1) must return float, got {type(_r).__name__}: {_r!r}"
_ledger.append(1)
assert abs(_r - 1.2) < 0.5, f"round(1.25, 1) must be near 1.2, got {_r!r}"
_ledger.append(1)

_r2 = round(3.14159, 2)
assert isinstance(_r2, float), f"round(3.14159, 2) must return float, got {type(_r2).__name__}: {_r2!r}"
_ledger.append(1)
assert abs(_r2 - 3.14) < 0.5, f"round(3.14159, 2) must be near 3.14, got {_r2!r}"
_ledger.append(1)

_r3 = round(0.123456, 3)
assert isinstance(_r3, float), f"round(0.123456, 3) must return float, got {type(_r3).__name__}: {_r3!r}"
_ledger.append(1)
assert abs(_r3 - 0.123) < 0.5, f"round(0.123456, 3) must be near 0.123, got {_r3!r}"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_float_int_special_value_silent {sum(_ledger)} asserts")
