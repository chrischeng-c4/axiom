# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_subscript_wrong_type_silent_none"
# subject = "cpython321.lang_subscript_wrong_type_silent_none"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_subscript_wrong_type_silent_none.py"
# status = "filled"
# ///
"""cpython321.lang_subscript_wrong_type_silent_none: execute CPython 3.12 seed lang_subscript_wrong_type_silent_none"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Spec seed for CPython TypeError contract on the wrong-type-
# subscript corners that mamba silently returns `None` from.
# Surface: CPython rejects any subscript `obj[i]` where `i` is not
# an integer (or a slice) on every sequence type — `str` / `bytes`
# / `list` / `tuple` all raise TypeError("indices must be integers
# or slices, not <type>") when handed a `float` / `str` / `None`
# index. Mamba silently returns `None` for the same expression,
# meaning downstream code that does e.g. `s[user_input]` silently
# no-ops on a non-integer index instead of failing loud. This is
# a high-impact silent-coercion class — sequence subscripting is
# one of the most-used Python operations.
#
# Probes (every form CPython rejects, mamba silently returns None):
#   • "hello"[1.5]              → mamba: None        (TypeError)
#   • b"hello"[1.5]             → mamba: None        (TypeError)
#   • [1, 2, 3][1.5]            → mamba: None        (TypeError)
#   • (1, 2, 3)[1.5]            → mamba: None        (TypeError)
#   • "hello"["1"]              → mamba: None        (TypeError)
#   • b"hello"["1"]             → mamba: None        (TypeError)
#   • [1, 2, 3]["1"]            → mamba: None        (TypeError)
#   • (1, 2, 3)["1"]            → mamba: None        (TypeError)
#   • "hello"[None]             → mamba: None        (TypeError)
#   • [1, 2, 3][None]           → mamba: None        (TypeError)
#   • (1, 2, 3)[None]           → mamba: None        (TypeError)
#
# CPython contract (uniform across every sequence type):
#   <sequence>[<non-int-non-slice>]
#       → TypeError("<type> indices must be integers or slices,
#                   not <bad-index-type>").
#
# `Any`-typed holders push the index past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so
# the runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_str: Any = "hello"
_bytes: Any = b"hello"
_list: Any = [1, 2, 3]
_tup: Any = (1, 2, 3)

_idx_float: Any = 1.5
_idx_str: Any = "1"
_idx_none: Any = None

# str[float]
try:
    _ = _str[_idx_float]
    raise AssertionError("'hello'[1.5] must raise TypeError")
except TypeError:
    _ledger.append(1)

# bytes[float]
try:
    _ = _bytes[_idx_float]
    raise AssertionError("b'hello'[1.5] must raise TypeError")
except TypeError:
    _ledger.append(1)

# list[float]
try:
    _ = _list[_idx_float]
    raise AssertionError("[1,2,3][1.5] must raise TypeError")
except TypeError:
    _ledger.append(1)

# tuple[float]
try:
    _ = _tup[_idx_float]
    raise AssertionError("(1,2,3)[1.5] must raise TypeError")
except TypeError:
    _ledger.append(1)

# str[str]
try:
    _ = _str[_idx_str]
    raise AssertionError("'hello'['1'] must raise TypeError")
except TypeError:
    _ledger.append(1)

# bytes[str]
try:
    _ = _bytes[_idx_str]
    raise AssertionError("b'hello'['1'] must raise TypeError")
except TypeError:
    _ledger.append(1)

# list[str]
try:
    _ = _list[_idx_str]
    raise AssertionError("[1,2,3]['1'] must raise TypeError")
except TypeError:
    _ledger.append(1)

# tuple[str]
try:
    _ = _tup[_idx_str]
    raise AssertionError("(1,2,3)['1'] must raise TypeError")
except TypeError:
    _ledger.append(1)

# str[None]
try:
    _ = _str[_idx_none]
    raise AssertionError("'hello'[None] must raise TypeError")
except TypeError:
    _ledger.append(1)

# list[None]
try:
    _ = _list[_idx_none]
    raise AssertionError("[1,2,3][None] must raise TypeError")
except TypeError:
    _ledger.append(1)

# tuple[None]
try:
    _ = _tup[_idx_none]
    raise AssertionError("(1,2,3)[None] must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_subscript_wrong_type_silent_none {sum(_ledger)} asserts")
