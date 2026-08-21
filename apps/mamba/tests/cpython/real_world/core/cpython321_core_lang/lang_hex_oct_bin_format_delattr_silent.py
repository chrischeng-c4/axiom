# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_hex_oct_bin_format_delattr_silent"
# subject = "cpython321.lang_hex_oct_bin_format_delattr_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_hex_oct_bin_format_delattr_silent.py"
# status = "filled"
# ///
"""cpython321.lang_hex_oct_bin_format_delattr_silent: execute CPython 3.12 seed lang_hex_oct_bin_format_delattr_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Spec seed for CPython TypeError / AttributeError contract on
# the numeric-repr / format-spec / attribute-deletion corners
# that mamba silently returns `None` / a default string / a no-op.
# Surface: CPython rejects (1) `hex(non_int)` / `oct(non_int)` /
# `bin(non_int)` because every one of those routes through
# `__index__` and non-integer types don't implement it —
# TypeError("'<type>' object cannot be interpreted as an integer");
# (2) `format(value, non_str_spec)` because the second argument
# must be `str` per `object.__format__(self, format_spec: str)` —
# TypeError("format() argument 2 must be str, not <type>"); (3)
# `delattr(builtin_container, name)` because built-in immutable
# containers (`dict`, `list`, `tuple`, `set`, `str`, `bytes`,
# `int`, `float`) don't allow arbitrary attribute deletion via
# `del obj.attr` — AttributeError("'<type>' object has no
# attribute '<name>'"). Mamba accepts every form and silently
# returns `None` / the unmodified value / a no-op, so code like
# `hex(meta.get("opaque_id"))` silently produces `None` rather
# than failing loud when `opaque_id` is a `str` instead of `int`,
# `format(value, fmt_spec_from_config)` silently ignores a
# non-string spec, and `delattr(cache, key)` silently leaves the
# cache untouched.
#
# Probes (every form CPython rejects, mamba silently coerces):
#   • hex('a')                       → mamba: None      (TypeError)
#   • hex(None)                      → mamba: None      (TypeError)
#   • hex([1,2])                     → mamba: None      (TypeError)
#   • hex(1.5)                       → mamba: None      (TypeError)
#   • oct('a')                       → mamba: None      (TypeError)
#   • oct(None)                      → mamba: None      (TypeError)
#   • oct([1,2])                     → mamba: None      (TypeError)
#   • oct(1.5)                       → mamba: None      (TypeError)
#   • bin('a')                       → mamba: None      (TypeError)
#   • bin(None)                      → mamba: None      (TypeError)
#   • bin([1,2])                     → mamba: None      (TypeError)
#   • bin(1.5)                       → mamba: None      (TypeError)
#   • format(1, [1,2])               → mamba: '1'       (TypeError)
#   • format(1, None)                → mamba: '1'       (TypeError)
#   • format(1, 99)                  → mamba: '1'       (TypeError)
#   • delattr({}, 'k')               → mamba: no-op     (AttributeError)
#   • delattr([], 'k')               → mamba: no-op     (AttributeError)
#
# CPython contract (uniform across every form):
#   hex/oct/bin(non_int)
#       → TypeError("'<type>' object cannot be interpreted as an
#                    integer");
#   format(value, non_str_spec)
#       → TypeError("format() argument 2 must be str, not <type>");
#   delattr(builtin_container, name)
#       → AttributeError("'<type>' object has no attribute '<name>'").
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so
# the runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_s: Any = "a"
_none: Any = None
_lst: Any = [1, 2]
_flt: Any = 1.5
_int: Any = 99
_dct: Any = {"existing": 1}
_lst_target: Any = [1, 2, 3]

# hex(str) — TypeError on CPython
try:
    _ = hex(_s)
    raise AssertionError("hex('a') must raise TypeError")
except TypeError:
    _ledger.append(1)

# hex(None)
try:
    _ = hex(_none)
    raise AssertionError("hex(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# hex(list)
try:
    _ = hex(_lst)
    raise AssertionError("hex([1,2]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# hex(float)
try:
    _ = hex(_flt)
    raise AssertionError("hex(1.5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# oct(str)
try:
    _ = oct(_s)
    raise AssertionError("oct('a') must raise TypeError")
except TypeError:
    _ledger.append(1)

# oct(None)
try:
    _ = oct(_none)
    raise AssertionError("oct(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# oct(list)
try:
    _ = oct(_lst)
    raise AssertionError("oct([1,2]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# oct(float)
try:
    _ = oct(_flt)
    raise AssertionError("oct(1.5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# bin(str)
try:
    _ = bin(_s)
    raise AssertionError("bin('a') must raise TypeError")
except TypeError:
    _ledger.append(1)

# bin(None)
try:
    _ = bin(_none)
    raise AssertionError("bin(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# bin(list)
try:
    _ = bin(_lst)
    raise AssertionError("bin([1,2]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# bin(float)
try:
    _ = bin(_flt)
    raise AssertionError("bin(1.5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# format(int, list-spec) — format_spec must be str
try:
    _ = format(1, _lst)
    raise AssertionError("format(1, [1,2]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# format(int, None-spec)
try:
    _ = format(1, _none)
    raise AssertionError("format(1, None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# format(int, int-spec)
try:
    _ = format(1, _int)
    raise AssertionError("format(1, 99) must raise TypeError")
except TypeError:
    _ledger.append(1)

# delattr(dict, 'k') — dict has no instance attribute named 'k'
try:
    delattr(_dct, "k")
    raise AssertionError("delattr({}, 'k') must raise AttributeError")
except AttributeError:
    _ledger.append(1)

# delattr(list, 'k') — list has no instance attribute named 'k'
try:
    delattr(_lst_target, "k")
    raise AssertionError("delattr([], 'k') must raise AttributeError")
except AttributeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_hex_oct_bin_format_delattr_silent {sum(_ledger)} asserts")
