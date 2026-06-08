# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_abs_round_complex_divmod_ord_wrong_type_silent"
# subject = "cpython321.lang_abs_round_complex_divmod_ord_wrong_type_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_abs_round_complex_divmod_ord_wrong_type_silent.py"
# status = "filled"
# ///
"""cpython321.lang_abs_round_complex_divmod_ord_wrong_type_silent: execute CPython 3.12 seed lang_abs_round_complex_divmod_ord_wrong_type_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Spec seed for CPython TypeError contract on the `abs(non_num)` /
# `round(non_num)` / `complex(non_str_non_num)` /
# `divmod(non_num, ...)` / `ord(non_str)` corners where mamba
# silently coerces to a default (`0`, `0j`, or `None`) instead of
# raising the canonical TypeError.
#
# Surface: CPython rejects (1) `abs(non_num)` for any operand whose
# type does not implement `__abs__` — `None`, `list`, `dict`,
# `bytes`, `tuple` — TypeError("bad operand type for abs():
# '<type>'"); (2) `round(non_num)` for any operand whose type does
# not implement `__round__` — `None`, `list`, `dict`, `bytes` —
# TypeError("type <type> doesn't define __round__ method"); (3)
# `complex(non_str_non_num)` because the first argument must be a
# string or a number — `list`, `dict`, `None`, `bytes`, `tuple` all
# rejected — TypeError("complex() first argument must be a string
# or a number, not '<type>'"); (4) `divmod(a, b)` when either
# operand lacks the `__divmod__` / `__rdivmod__` protocol — `None`,
# `list`, `str`, `tuple` — TypeError("unsupported operand type(s)
# for divmod(): '<a>' and '<b>'"); (5) `ord(non_str)` because
# `ord()` requires a one-character string — `list`, `None`, `int`,
# `dict` — TypeError("ord() expected string of length 1, but
# <type> found").
#
# Mamba accepts every form and silently returns `0` for `abs()` /
# `round()`, `0j` for `complex()`, `None` for `divmod()` / `ord()`,
# so code like `total = abs(maybe_value)` where `maybe_value`
# accidentally arrived as `None` (e.g. an upstream lookup that
# returned `None` for the missing entry) silently produces `0`,
# masking a missing-value bug as a clean zero. Similarly,
# `divmod(payload, chunk_size)` where `payload` was an
# accidentally-unparsed `bytes` blob silently returns `None`,
# producing a downstream "NoneType is not iterable" instead of the
# canonical TypeError that names the actual problem operand.
#
# Existing lang_typeerror_builtin_silent_coerce.py covers
# `abs(str)` / `round(str)` / `pow(str, str)` and existing
# lang_unicode_chr_ord_decompress_silent.py covers `ord(empty)` /
# `ord(multichar)` (the length-1 check). Existing
# lang_typeerror_unhashable.py covers `hash(unhashable)`. This seed
# covers the FRESH divergence family — `abs` / `round` /
# `complex` / `divmod` / `ord` with WRONG-TYPE arguments (not
# wrong-length / wrong-content), which all route through the
# `__abs__` / `__round__` / `__index__` / `__divmod__` /
# str-protocol checks at the SAME runtime level but mamba
# silently coerces.
#
# Probes (every form CPython rejects, mamba silently coerces):
#   • abs(None)                      → mamba: 0          (TypeError)
#   • abs([1,2])                     → mamba: 0          (TypeError)
#   • abs({1:2})                     → mamba: 0          (TypeError)
#   • abs(b'abc')                    → mamba: 0          (TypeError)
#   • abs((1,2))                     → mamba: 0          (TypeError)
#   • round(None)                    → mamba: 0          (TypeError)
#   • round([1,2])                   → mamba: 0          (TypeError)
#   • round({1:2})                   → mamba: 0          (TypeError)
#   • round(b'abc')                  → mamba: 0          (TypeError)
#   • complex([1,2])                 → mamba: 0j         (TypeError)
#   • complex({1:2})                 → mamba: 0j         (TypeError)
#   • complex(None)                  → mamba: 0j         (TypeError)
#   • complex(b'1+2j')               → mamba: 0j         (TypeError)
#   • complex((1,2))                 → mamba: 0j         (TypeError)
#   • divmod(None, 2)                → mamba: None       (TypeError)
#   • divmod(2, None)                → mamba: None       (TypeError)
#   • divmod([1], 2)                 → mamba: None       (TypeError)
#   • divmod('a', 'b')               → mamba: None       (TypeError)
#   • divmod((1,), (2,))             → mamba: None       (TypeError)
#   • ord([1,2])                     → mamba: None       (TypeError)
#   • ord(None)                      → mamba: None       (TypeError)
#   • ord(5)                         → mamba: None       (TypeError)
#   • ord({1:2})                     → mamba: None       (TypeError)
#
# CPython contract (uniform across every form):
#   abs(non_num)
#       → TypeError("bad operand type for abs(): '<type>'");
#   round(non_num)
#       → TypeError("type <type> doesn't define __round__ method");
#   complex(non_str_non_num)
#       → TypeError("complex() first argument must be a string or
#                    a number, not '<type>'");
#   divmod(a, b) where neither side defines __divmod__
#       → TypeError("unsupported operand type(s) for divmod():
#                    '<a>' and '<b>'");
#   ord(non_str_or_len!=1)
#       → TypeError("ord() expected string of length 1, but
#                    <type> found").
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so
# the runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_n: Any = None
_l: Any = [1, 2]
_d: Any = {1: 2}
_b: Any = b'abc'
_t: Any = (1, 2)
_i: Any = 5
_s: Any = 'abc'

# abs(None) — NoneType has no __abs__
try:
    _ = abs(_n)
    raise AssertionError("abs(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# abs([1, 2]) — list has no __abs__
try:
    _ = abs(_l)
    raise AssertionError("abs([1,2]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# abs({1: 2}) — dict has no __abs__
try:
    _ = abs(_d)
    raise AssertionError("abs({1:2}) must raise TypeError")
except TypeError:
    _ledger.append(1)

# abs(b'abc') — bytes has no __abs__
try:
    _ = abs(_b)
    raise AssertionError("abs(b'abc') must raise TypeError")
except TypeError:
    _ledger.append(1)

# abs((1, 2)) — tuple has no __abs__
try:
    _ = abs(_t)
    raise AssertionError("abs((1,2)) must raise TypeError")
except TypeError:
    _ledger.append(1)

# round(None) — NoneType has no __round__
try:
    _ = round(_n)
    raise AssertionError("round(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# round([1, 2]) — list has no __round__
try:
    _ = round(_l)
    raise AssertionError("round([1,2]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# round({1: 2}) — dict has no __round__
try:
    _ = round(_d)
    raise AssertionError("round({1:2}) must raise TypeError")
except TypeError:
    _ledger.append(1)

# round(b'abc') — bytes has no __round__
try:
    _ = round(_b)
    raise AssertionError("round(b'abc') must raise TypeError")
except TypeError:
    _ledger.append(1)

# complex([1, 2]) — list is not str or number
try:
    _ = complex(_l)
    raise AssertionError("complex([1,2]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# complex({1: 2}) — dict is not str or number
try:
    _ = complex(_d)
    raise AssertionError("complex({1:2}) must raise TypeError")
except TypeError:
    _ledger.append(1)

# complex(None)
try:
    _ = complex(_n)
    raise AssertionError("complex(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# complex(b'1+2j') — bytes is not str or number
try:
    _ = complex(_b)
    raise AssertionError("complex(b'1+2j') must raise TypeError")
except TypeError:
    _ledger.append(1)

# complex((1, 2)) — tuple is not str or number
try:
    _ = complex(_t)
    raise AssertionError("complex((1,2)) must raise TypeError")
except TypeError:
    _ledger.append(1)

# divmod(None, 2) — NoneType lacks __divmod__
try:
    _ = divmod(_n, 2)
    raise AssertionError("divmod(None, 2) must raise TypeError")
except TypeError:
    _ledger.append(1)

# divmod(2, None) — NoneType lacks __rdivmod__
try:
    _ = divmod(2, _n)
    raise AssertionError("divmod(2, None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# divmod([1], 2) — list lacks __divmod__
try:
    _ = divmod(_l, 2)
    raise AssertionError("divmod([1,2], 2) must raise TypeError")
except TypeError:
    _ledger.append(1)

# divmod('a', 'b') — str lacks __divmod__
try:
    _ = divmod(_s, _s)
    raise AssertionError("divmod('a', 'b') must raise TypeError")
except TypeError:
    _ledger.append(1)

# divmod((1,), (2,)) — tuple lacks __divmod__
try:
    _ = divmod(_t, _t)
    raise AssertionError("divmod((1,), (2,)) must raise TypeError")
except TypeError:
    _ledger.append(1)

# ord([1, 2]) — list is not a one-char string
try:
    _ = ord(_l)
    raise AssertionError("ord([1,2]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# ord(None)
try:
    _ = ord(_n)
    raise AssertionError("ord(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# ord(5) — int is not a one-char string
try:
    _ = ord(_i)
    raise AssertionError("ord(5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# ord({1: 2}) — dict is not a one-char string
try:
    _ = ord(_d)
    raise AssertionError("ord({1:2}) must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_abs_round_complex_divmod_ord_wrong_type_silent {sum(_ledger)} asserts")
