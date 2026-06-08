# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_isinstance_int_bytes_arg_silent"
# subject = "cpython321.lang_isinstance_int_bytes_arg_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_isinstance_int_bytes_arg_silent.py"
# status = "filled"
# ///
"""cpython321.lang_isinstance_int_bytes_arg_silent: execute CPython 3.12 seed lang_isinstance_int_bytes_arg_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Spec seed for CPython TypeError / ValueError contract on the
# argument-validation corners of `isinstance` / `issubclass` and the
# integer ↔ bytes converters (`int.to_bytes` / `int.from_bytes`).
# Surface: CPython rejects (1) `isinstance(_, non_class)` /
# `issubclass(non_class, _)` because the second/first argument must
# be a real class (or a tuple of classes / Union, not a value, str, or
# list of classes) — TypeError; (2) `int.to_bytes(-1, ...)` because
# the requested byte-buffer length must be non-negative — ValueError;
# (3) `int.to_bytes(..., "midget")` and `int.from_bytes(..., "midget")`
# because the byteorder must be exactly the string `'little'` or
# `'big'` — ValueError. Mamba 0.3.60 silently returns `False` /
# `True` / `b''` / `b'\x05\x00'` / `1280` instead of dispatching the
# argument-validator → TypeError / ValueError.
#
# Probes (every form CPython rejects, mamba silently coerces):
#   • isinstance(5, 5)               → mamba: False (TypeError)
#   • isinstance(5, None)            → mamba: False (TypeError)
#   • isinstance(5, "int")           → mamba: True  (TypeError)
#   • isinstance(5, [int, str])      → mamba: False (TypeError)
#   • issubclass(5, int)             → mamba: False (TypeError)
#   • issubclass(int, None)          → mamba: False (TypeError)
#   • issubclass(int, "int")         → mamba: True  (TypeError)
#   • issubclass(int, [int, str])    → mamba: False (TypeError)
#   • (5).to_bytes(-1, "big")        → mamba: b''   (ValueError)
#   • (5).to_bytes(2, "midget")      → mamba: b'\x05\x00' (ValueError)
#   • int.from_bytes(b'\x00\x05', "midget")
#                                    → mamba: 1280  (ValueError)
#
# CPython contract:
#   isinstance(_, non_class)
#                          → TypeError("isinstance() arg 2 must be a
#                                  type, a tuple of types, or a
#                                  union");
#   issubclass(non_class, _)
#                          → TypeError("issubclass() arg 1 must be a
#                                  class");
#   issubclass(_, non_class)
#                          → TypeError("issubclass() arg 2 must be a
#                                  class, a tuple of classes, or a
#                                  union");
#   int.to_bytes(neg, ...)
#                          → ValueError("length argument must be
#                                  non-negative");
#   int.to_bytes(_, bad_order)
#   int.from_bytes(_, bad_order)
#                          → ValueError("byteorder must be either
#                                  'little' or 'big'").
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so the
# runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_o: Any = 5
_n: Any = None
_str_class: Any = "int"
_intval: Any = 5
_lst_classes: Any = [int, str]
_neg1: Any = -1
_bad_order: Any = "midget"
_b: Any = b"\x00\x05"

# isinstance(_, 5) — arg 2 is not a class
try:
    _ = isinstance(_o, _intval)
    raise AssertionError("isinstance(_, int_value) must raise TypeError")
except TypeError:
    _ledger.append(1)

# isinstance(_, None) — arg 2 is None
try:
    _ = isinstance(_o, _n)
    raise AssertionError("isinstance(_, None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# isinstance(_, "int") — arg 2 is a string
try:
    _ = isinstance(_o, _str_class)
    raise AssertionError("isinstance(_, str) must raise TypeError")
except TypeError:
    _ledger.append(1)

# isinstance(_, [int, str]) — arg 2 is a list (must be tuple, not list)
try:
    _ = isinstance(_o, _lst_classes)
    raise AssertionError("isinstance(_, list_of_classes) must raise TypeError")
except TypeError:
    _ledger.append(1)

# issubclass(5, int) — arg 1 must be a class
try:
    _ = issubclass(_intval, int)
    raise AssertionError("issubclass(int_value, int) must raise TypeError")
except TypeError:
    _ledger.append(1)

# issubclass(int, None) — arg 2 must be a class
try:
    _ = issubclass(int, _n)
    raise AssertionError("issubclass(int, None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# issubclass(int, "int") — arg 2 is a string
try:
    _ = issubclass(int, _str_class)
    raise AssertionError("issubclass(int, str) must raise TypeError")
except TypeError:
    _ledger.append(1)

# issubclass(int, [int, str]) — arg 2 is a list
try:
    _ = issubclass(int, _lst_classes)
    raise AssertionError("issubclass(int, list_of_classes) must raise TypeError")
except TypeError:
    _ledger.append(1)

# int.to_bytes(-1, "big") — length must be non-negative
try:
    _ = (5).to_bytes(_neg1, "big")
    raise AssertionError("int.to_bytes(-1, ...) must raise ValueError")
except ValueError:
    _ledger.append(1)

# int.to_bytes(2, "midget") — byteorder must be 'little' or 'big'
try:
    _ = (5).to_bytes(2, _bad_order)
    raise AssertionError("int.to_bytes(_, bad_order) must raise ValueError")
except ValueError:
    _ledger.append(1)

# int.from_bytes(b'\x00\x05', "midget") — byteorder must be 'little' or 'big'
try:
    _ = int.from_bytes(_b, _bad_order)
    raise AssertionError("int.from_bytes(_, bad_order) must raise ValueError")
except ValueError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_isinstance_int_bytes_arg_silent {sum(_ledger)} asserts")
