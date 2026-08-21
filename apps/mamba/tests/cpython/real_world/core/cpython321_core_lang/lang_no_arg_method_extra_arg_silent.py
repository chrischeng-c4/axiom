# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_no_arg_method_extra_arg_silent"
# subject = "cpython321.lang_no_arg_method_extra_arg_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_no_arg_method_extra_arg_silent.py"
# status = "filled"
# ///
"""cpython321.lang_no_arg_method_extra_arg_silent: execute CPython 3.12 seed lang_no_arg_method_extra_arg_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Spec seed for CPython TypeError contract on the
# `no_arg_method(extra_arg)` / `fixed_arity_method(extras)` corners
# where mamba silently no-ops the extra argument and returns the
# no-arg result instead of raising the canonical TypeError.
#
# Surface: CPython rejects (1) `str.lower(extra)` / `str.upper(extra)`
# / `str.title(extra)` / `str.capitalize(extra)` / `str.swapcase(extra)`
# / `str.casefold(extra)` — TypeError("str.<method>() takes no
# arguments (1 given)"); (2) `list.copy(extra)` / `list.reverse(extra)`
# / `list.clear(extra)` — TypeError("list.<method>() takes no
# arguments (1 given)"); (3) `dict.copy(extra)` / `dict.keys(extra)`
# / `dict.values(extra)` / `dict.items(extra)` / `dict.clear(extra)`
# — TypeError("dict.<method>() takes no arguments (1 given)"); (4)
# `tuple.count(value, extra)` — TypeError("tuple.count() takes
# exactly one argument (2 given)"); (5) `int.bit_length(extra)`
# / `int.bit_count(extra)` — TypeError("int.<method>() takes no
# arguments (1 given)"); (6) `bytes.hex(sep, bytes_per_sep, extra)`
# — TypeError("hex() takes at most 2 arguments (3 given)").
#
# Mamba accepts every form and silently ignores the extra
# argument, returning the no-arg / canonical-arity result. So code
# like `total_unique = items.count(needle, MAX_DEPTH)` — where the
# caller accidentally passed a recursion-depth parameter to a
# tuple.count call expecting only one argument — silently returns
# the raw count (ignoring MAX_DEPTH), masking the call-site bug as
# a plausible value. Similarly, `key.bit_length(width_hint)`
# silently returns the bit length of the key (ignoring the
# width_hint) instead of the canonical "takes no arguments"
# TypeError that names the call-site problem.
#
# Existing lang_typeerror_call_arity.py covers the BUILT-IN
# arity-mismatch family — `len()` / `abs()` / `ord()` / `chr()`
# with wrong arity. Existing lang_typeerror_str_bytes_method_silent.py
# covers `str.<method>()` with WRONG-TYPE arguments. Existing
# lang_str_method_width_index_wrong_type_silent.py covers
# `str.<width_method>(non_int)`. This seed covers the FRESH
# divergence family — the METHOD-side arity-mismatch
# (where the method itself takes a fixed number of arguments and
# mamba silently accepts MORE than that, ignoring the trailing
# args).
#
# Probes (every form CPython rejects, mamba silently no-ops):
#   • 'abc'.lower(99)            → mamba: 'abc'   (TypeError)
#   • 'abc'.upper(99)            → mamba: 'ABC'   (TypeError)
#   • 'abc'.title(99)            → mamba: 'Abc'   (TypeError)
#   • 'abc'.capitalize(99)       → mamba: 'Abc'   (TypeError)
#   • 'abc'.swapcase(99)         → mamba: 'ABC'   (TypeError)
#   • 'abc'.casefold(99)         → mamba: 'abc'   (TypeError)
#   • [1,2,3].copy(99)           → mamba: [1,2,3] (TypeError)
#   • [1,2,3].reverse(99)        → mamba: None    (TypeError)
#   • [1,2,3].clear(99)          → mamba: None    (TypeError)
#   • {1:2}.copy(99)             → mamba: {1:2}   (TypeError)
#   • {1:2}.keys(99)             → mamba: [1]     (TypeError)
#   • {1:2}.values(99)           → mamba: [2]     (TypeError)
#   • {1:2}.items(99)            → mamba: [(1,2)] (TypeError)
#   • {1:2}.clear(99)            → mamba: None    (TypeError)
#   • (1,2,3).count(1, 99)       → mamba: 1       (TypeError)
#   • (5).bit_length(99)         → mamba: 3       (TypeError)
#   • (5).bit_count(99)          → mamba: 2       (TypeError)
#   • b'abc'.hex(99, 99, 99)     → mamba: '616263'(TypeError)
#
# CPython contract (uniform across every form):
#   str.<no_arg_method>(extra)
#       → TypeError("str.<method>() takes no arguments (1 given)");
#   list.<no_arg_method>(extra) / dict.<no_arg_method>(extra)
#       → TypeError("<type>.<method>() takes no arguments (1 given)");
#   tuple.count(value, extra)
#       → TypeError("tuple.count() takes exactly one argument
#                    (2 given)");
#   int.<no_arg_method>(extra)
#       → TypeError("int.<method>() takes no arguments (1 given)");
#   bytes.hex(sep, bytes_per_sep, extra)
#       → TypeError("hex() takes at most 2 arguments (3 given)").
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so
# the runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_s: Any = 'abc'
_l: Any = [1, 2, 3]
_d: Any = {1: 2}
_t: Any = (1, 2, 3)
_i: Any = 5
_b: Any = b'abc'

# 'abc'.lower(99) — no-arg method, extra arg
try:
    _ = _s.lower(99)
    raise AssertionError("'abc'.lower(99) must raise TypeError")
except TypeError:
    _ledger.append(1)

# 'abc'.upper(99)
try:
    _ = _s.upper(99)
    raise AssertionError("'abc'.upper(99) must raise TypeError")
except TypeError:
    _ledger.append(1)

# 'abc'.title(99)
try:
    _ = _s.title(99)
    raise AssertionError("'abc'.title(99) must raise TypeError")
except TypeError:
    _ledger.append(1)

# 'abc'.capitalize(99)
try:
    _ = _s.capitalize(99)
    raise AssertionError("'abc'.capitalize(99) must raise TypeError")
except TypeError:
    _ledger.append(1)

# 'abc'.swapcase(99)
try:
    _ = _s.swapcase(99)
    raise AssertionError("'abc'.swapcase(99) must raise TypeError")
except TypeError:
    _ledger.append(1)

# 'abc'.casefold(99)
try:
    _ = _s.casefold(99)
    raise AssertionError("'abc'.casefold(99) must raise TypeError")
except TypeError:
    _ledger.append(1)

# [1,2,3].copy(99) — no-arg method, extra arg
try:
    _ = _l.copy(99)
    raise AssertionError("[1,2,3].copy(99) must raise TypeError")
except TypeError:
    _ledger.append(1)

# [1,2,3].reverse(99)
try:
    _ = _l.reverse(99)
    raise AssertionError("[1,2,3].reverse(99) must raise TypeError")
except TypeError:
    _ledger.append(1)

# [1,2,3].clear(99)
try:
    _ = _l.clear(99)
    raise AssertionError("[1,2,3].clear(99) must raise TypeError")
except TypeError:
    _ledger.append(1)

# {1:2}.copy(99)
try:
    _ = _d.copy(99)
    raise AssertionError("{1:2}.copy(99) must raise TypeError")
except TypeError:
    _ledger.append(1)

# {1:2}.keys(99)
try:
    _ = _d.keys(99)
    raise AssertionError("{1:2}.keys(99) must raise TypeError")
except TypeError:
    _ledger.append(1)

# {1:2}.values(99)
try:
    _ = _d.values(99)
    raise AssertionError("{1:2}.values(99) must raise TypeError")
except TypeError:
    _ledger.append(1)

# {1:2}.items(99)
try:
    _ = _d.items(99)
    raise AssertionError("{1:2}.items(99) must raise TypeError")
except TypeError:
    _ledger.append(1)

# {1:2}.clear(99)
try:
    _ = _d.clear(99)
    raise AssertionError("{1:2}.clear(99) must raise TypeError")
except TypeError:
    _ledger.append(1)

# (1,2,3).count(1, 99) — fixed-1 method, extra arg
try:
    _ = _t.count(1, 99)
    raise AssertionError("(1,2,3).count(1, 99) must raise TypeError")
except TypeError:
    _ledger.append(1)

# (5).bit_length(99) — no-arg method, extra arg
try:
    _ = _i.bit_length(99)
    raise AssertionError("(5).bit_length(99) must raise TypeError")
except TypeError:
    _ledger.append(1)

# (5).bit_count(99)
try:
    _ = _i.bit_count(99)
    raise AssertionError("(5).bit_count(99) must raise TypeError")
except TypeError:
    _ledger.append(1)

# b'abc'.hex(99, 99, 99) — at-most-2-args method, third extra arg
try:
    _ = _b.hex(99, 99, 99)
    raise AssertionError("b'abc'.hex(99,99,99) must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_no_arg_method_extra_arg_silent {sum(_ledger)} asserts")
