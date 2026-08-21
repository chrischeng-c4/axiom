# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_immutable_mutation_silent"
# subject = "cpython321.lang_immutable_mutation_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_immutable_mutation_silent.py"
# status = "filled"
# ///
"""cpython321.lang_immutable_mutation_silent: execute CPython 3.12 seed lang_immutable_mutation_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Spec seed for CPython AttributeError / IndexError / KeyError /
# TypeError contract on the immutable-mutation and out-of-range-
# delete corners that mamba silently accepts. Surface: CPython
# rejects (1) `int_obj.x = v` / `float_obj.x = v` / `str_obj.x = v`
# / `tuple_obj.x = v` / `bool_obj.x = v` / `bytes_obj.x = v` /
# `frozenset_obj.x = v` because immutable scalars have no `__dict__`
# and their type does not allow new attributes — AttributeError
# (e.g. "'int' object has no attribute 'x'"), not silent assignment
# that pretends to land somewhere; (2) `del list[oor_index]` /
# `del list[-oor_index]` because the index is out of range —
# IndexError("list assignment index out of range"), not silent
# success that leaves the list unchanged; (3) `del dict[missing]`
# because the key isn't there — KeyError, not silent success; (4)
# `del tuple[i]` / `del str[i]` because the type doesn't support
# item deletion — TypeError, not silent success; (5) `str[i] = v`
# / `tuple[i] = v` / `bytes[i] = v` because the type doesn't support
# item assignment — TypeError, not silent success that leaves the
# original unchanged. Existing `lang_*_silent` seeds touch related
# coercion / coerce-to-None corners but the immutable-mutation
# family hasn't been pinned yet.
#
# Probes (every form CPython rejects, mamba silently accepts):
#   • i.x = 5       (i: int)              → mamba: ok (AttributeError)
#   • f.x = 5       (f: float)            → mamba: ok (AttributeError)
#   • s.x = 5       (s: str)              → mamba: ok (AttributeError)
#   • t.x = 5       (t: tuple)            → mamba: ok (AttributeError)
#   • bo.x = 5      (bo: bool)            → mamba: ok (AttributeError)
#   • b.x = 5       (b: bytes)            → mamba: ok (AttributeError)
#   • fs.x = 5      (fs: frozenset)       → mamba: ok (AttributeError)
#   • del L[100]    (L: list[int*3])      → mamba: ok (IndexError)
#   • del L[-100]   (L: list[int*3])      → mamba: ok (IndexError)
#   • del D[99]     (D: dict[1: ...])     → mamba: ok (KeyError)
#   • del T[0]      (T: tuple)            → mamba: ok (TypeError)
#   • del S[0]      (S: str)              → mamba: ok (TypeError)
#   • S[0] = "Z"    (S: str)              → mamba: ok (TypeError)
#   • T[0] = 99     (T: tuple)            → mamba: ok (TypeError)
#   • B[0] = 99     (B: bytes)            → mamba: ok (TypeError)
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so the
# runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_int: Any = 5
_float: Any = 1.5
_str: Any = "abc"
_tup: Any = (1, 2, 3)
_bool: Any = True
_bytes: Any = b"abc"
_fset: Any = frozenset([1, 2, 3])

# i.x = 5 — int has no __dict__
try:
    _int.x = 5
    raise AssertionError("int_obj.x = v must raise AttributeError")
except AttributeError:
    _ledger.append(1)

# f.x = 5 — float has no __dict__
try:
    _float.x = 5
    raise AssertionError("float_obj.x = v must raise AttributeError")
except AttributeError:
    _ledger.append(1)

# s.x = 5 — str has no __dict__
try:
    _str.x = 5
    raise AssertionError("str_obj.x = v must raise AttributeError")
except AttributeError:
    _ledger.append(1)

# t.x = 5 — tuple has no __dict__
try:
    _tup.x = 5
    raise AssertionError("tuple_obj.x = v must raise AttributeError")
except AttributeError:
    _ledger.append(1)

# bo.x = 5 — bool has no __dict__
try:
    _bool.x = 5
    raise AssertionError("bool_obj.x = v must raise AttributeError")
except AttributeError:
    _ledger.append(1)

# b.x = 5 — bytes has no __dict__
try:
    _bytes.x = 5
    raise AssertionError("bytes_obj.x = v must raise AttributeError")
except AttributeError:
    _ledger.append(1)

# fs.x = 5 — frozenset has no __dict__
try:
    _fset.x = 5
    raise AssertionError("frozenset_obj.x = v must raise AttributeError")
except AttributeError:
    _ledger.append(1)

# del L[100] — index out of range
_L: Any = [1, 2, 3]
try:
    del _L[100]
    raise AssertionError("del L[100] must raise IndexError")
except IndexError:
    _ledger.append(1)

# del L[-100] — negative index out of range
_L2: Any = [1, 2, 3]
try:
    del _L2[-100]
    raise AssertionError("del L[-100] must raise IndexError")
except IndexError:
    _ledger.append(1)

# del D[missing] — key not in dict
_D: Any = {1: "a"}
try:
    del _D[99]
    raise AssertionError("del D[99] must raise KeyError")
except KeyError:
    _ledger.append(1)

# del T[0] — tuple doesn't support deletion
_T: Any = (1, 2, 3)
try:
    del _T[0]
    raise AssertionError("del T[0] must raise TypeError")
except TypeError:
    _ledger.append(1)

# del S[0] — str doesn't support deletion
_S_del: Any = "abc"
try:
    del _S_del[0]
    raise AssertionError("del S[0] must raise TypeError")
except TypeError:
    _ledger.append(1)

# S[0] = "Z" — str is immutable
_S_set: Any = "abc"
try:
    _S_set[0] = "Z"
    raise AssertionError("S[0] = 'Z' must raise TypeError")
except TypeError:
    _ledger.append(1)

# T[0] = 99 — tuple is immutable
_T_set: Any = (1, 2, 3)
try:
    _T_set[0] = 99
    raise AssertionError("T[0] = 99 must raise TypeError")
except TypeError:
    _ledger.append(1)

# B[0] = 99 — bytes is immutable (bytearray is the mutable version)
_B_set: Any = b"abc"
try:
    _B_set[0] = 99
    raise AssertionError("B[0] = 99 must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_immutable_mutation_silent {sum(_ledger)} asserts")
