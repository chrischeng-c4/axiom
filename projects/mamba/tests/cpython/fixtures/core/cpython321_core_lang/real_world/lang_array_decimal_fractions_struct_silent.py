# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_array_decimal_fractions_struct_silent"
# subject = "cpython321.lang_array_decimal_fractions_struct_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_array_decimal_fractions_struct_silent.py"
# status = "filled"
# ///
"""cpython321.lang_array_decimal_fractions_struct_silent: execute CPython 3.12 seed lang_array_decimal_fractions_struct_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences across the
# array / decimal / fractions instance surface + the struct
# string-format directive + the decimal module-level context
# helpers pinned by atomic 178: `array` (the documented
# `array.array(typecode, iterable)` constructor instance class
# identity + len + subscript surface), `struct` (the documented
# `>3s` / `3s` string-format directive contract), `decimal`
# (the documented `Decimal(str)` constructor instance class
# identity + str repr + arithmetic + the documented `getcontext`
# / `setcontext` / `Context` / `ROUND_HALF_UP` /
# `ROUND_HALF_EVEN` / `InvalidOperation` module-level helper
# / class identifier surface), and `fractions` (the documented
# `Fraction(num, den)` constructor instance class identity +
# str repr + arithmetic surface).
#
# The matching subset (struct big-endian int / multi-int /
# multi-float pack-unpack + calcsize + module hasattr surface,
# statistics full central-tendency + dispersion + alternative-
# mean + positional-median float-cast value contract + module
# hasattr surface, array module hasattr surface (array /
# ArrayType / typecodes), decimal module hasattr Decimal only,
# fractions module hasattr Fraction only) is covered by
# `test_struct_statistics_module_hasattr_value_ops`; this
# fixture pins the CPython-only contracts that mamba currently
# elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • type(array.array('i', [1, 2, 3, 4, 5])).__name__ ==
#     "array" — documented constructor class identity (mamba:
#     returns "int" — array.array(...) produces a bare int
#     handle not the documented array instance);
#   • len(array.array('i', [1, 2, 3, 4, 5])) == 5 — documented
#     iterable-init len contract (mamba: 0 — the array
#     instance never receives the iterable);
#   • array.array('i', [1, 2, 3, 4, 5])[0] == 1 — documented
#     subscript instance method (mamba: returns None);
#   • struct.pack("3s", b"abc").hex() == "616263" — documented
#     string-format directive (mamba: returns "610000" —
#     only the first byte is packed and the rest are
#     zero-filled);
#   • type(decimal.Decimal("3.14")).__name__ == "Decimal" —
#     documented constructor class identity (mamba: returns
#     "int" — Decimal(...) produces an integer handle not
#     the documented Decimal instance);
#   • str(decimal.Decimal("3.14")) == "3.14" — documented
#     str repr instance contract (mamba: returns
#     "70368744177664" — the Decimal instance is broken and
#     str returns the raw integer handle);
#   • decimal.Decimal("1.1") + decimal.Decimal("2.2") ==
#     decimal.Decimal("3.3") — documented arithmetic
#     instance contract (mamba: returns negative garbage
#     int — Decimal arithmetic is broken);
#   • hasattr(decimal, "getcontext") is True — documented
#     module-level helper (mamba: False);
#   • hasattr(decimal, "setcontext") is True — documented
#     module-level helper (mamba: False);
#   • hasattr(decimal, "Context") is True — documented
#     class identifier (mamba: False);
#   • hasattr(decimal, "ROUND_HALF_UP") is True — documented
#     rounding-mode constant (mamba: False);
#   • hasattr(decimal, "ROUND_HALF_EVEN") is True —
#     documented rounding-mode constant (mamba: False);
#   • hasattr(decimal, "InvalidOperation") is True —
#     documented exception class (mamba: False);
#   • type(fractions.Fraction(1, 3)).__name__ == "Fraction"
#     — documented constructor class identity (mamba:
#     returns "int" — Fraction(...) produces an integer
#     handle not the documented Fraction instance);
#   • str(fractions.Fraction(1, 3)) == "1/3" — documented
#     str repr instance contract (mamba: returns
#     "1099511627776" — the Fraction instance is broken
#     and str returns the raw integer handle).
import array as _array_mod
import struct as _struct_mod
import decimal as _decimal_mod
import fractions as _fractions_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# constructors / instance attributes / module-level helpers
# that mamba's bundled type stubs do not surface accurately.
array: Any = _array_mod
struct: Any = _struct_mod
decimal: Any = _decimal_mod
fractions: Any = _fractions_mod


_ledger: list[int] = []

# 1) array.array — constructor class identity + len + [0]
_a = array.array('i', [1, 2, 3, 4, 5])
assert type(_a).__name__ == "array"; _ledger.append(1)
assert len(_a) == 5; _ledger.append(1)
assert _a[0] == 1; _ledger.append(1)

# 2) struct.pack — `3s` string-format directive
assert struct.pack("3s", b"abc").hex() == "616263"; _ledger.append(1)

# 3) decimal.Decimal — constructor class identity + str repr
_d = decimal.Decimal("3.14")
assert type(_d).__name__ == "Decimal"; _ledger.append(1)
assert str(_d) == "3.14"; _ledger.append(1)

# 4) decimal.Decimal — arithmetic value contract
assert decimal.Decimal("1.1") + decimal.Decimal("2.2") == decimal.Decimal("3.3"); _ledger.append(1)

# 5) decimal — module-level helper / class identifier surface
assert hasattr(decimal, "getcontext") == True; _ledger.append(1)
assert hasattr(decimal, "setcontext") == True; _ledger.append(1)
assert hasattr(decimal, "Context") == True; _ledger.append(1)
assert hasattr(decimal, "ROUND_HALF_UP") == True; _ledger.append(1)
assert hasattr(decimal, "ROUND_HALF_EVEN") == True; _ledger.append(1)
assert hasattr(decimal, "InvalidOperation") == True; _ledger.append(1)

# 6) fractions.Fraction — constructor class identity + str repr
_f = fractions.Fraction(1, 3)
assert type(_f).__name__ == "Fraction"; _ledger.append(1)
assert str(_f) == "1/3"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_array_decimal_fractions_struct_silent {sum(_ledger)} asserts")
