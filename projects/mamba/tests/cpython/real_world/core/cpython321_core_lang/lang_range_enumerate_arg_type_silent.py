# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_range_enumerate_arg_type_silent"
# subject = "cpython321.lang_range_enumerate_arg_type_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_range_enumerate_arg_type_silent.py"
# status = "filled"
# ///
"""cpython321.lang_range_enumerate_arg_type_silent: execute CPython 3.12 seed lang_range_enumerate_arg_type_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Spec seed for CPython TypeError contract on the `range(non_int)`
# / `enumerate(it, start=non_int)` corners where mamba silently
# coerces the non-int argument to a default (empty range / start=0)
# instead of raising the canonical "object cannot be interpreted as
# an integer" TypeError.
#
# Surface: CPython rejects (1) `range(non_int)` because every
# positional argument routes through `__index__` and float / str /
# None / list / dict / tuple don't implement it — TypeError("'<type>'
# object cannot be interpreted as an integer"); (2) `range(start,
# stop, non_int_step)` because the step argument also routes
# through `__index__` — TypeError("'<type>' object cannot be
# interpreted as an integer"); (3) `enumerate(it, start=non_int)`
# because the `start` parameter routes through `__index__` —
# TypeError("'<type>' object cannot be interpreted as an integer").
#
# Mamba accepts every form and silently:
#   - returns the empty range `[]` for `range(non_int_stop)` —
#     masking the operator's intent to iterate the count;
#   - ignores the step argument for `range(0, N, non_int_step)`,
#     returning the default-step result — so
#     `range(0, 5, "abc") == range(0, 5)` and the operator's
#     intent to skip elements is lost;
#   - ignores the start argument for `enumerate(it, non_int_start)`,
#     returning the default-start (0) tuples — so
#     `enumerate(["a","b"], "abc") == enumerate(["a","b"])` and the
#     operator's intent to offset the index is lost.
#
# Existing lang_codec_conv_arg_type_silent.py covers
# `bin/hex/oct(non_int)` and `int(s, non_int_base)` — the
# DIRECT-INDEX-ARG variants on conversion builtins. Existing
# lang_index_codec_chr_silent.py covers `chr(non_int)` — the
# UNICODE-codepoint variant. Existing
# lang_slice_range_arg_silent.py covers `range(stop) with stop=()`
# / `slice(stop, non_int)` — the ARGUMENT-COUNT / SLICE-ATTR
# variants. This seed covers the FRESH divergence family — the
# `range(non_int_pos_arg)` / `range(start, stop, non_int_step)` /
# `enumerate(it, start=non_int)` family where the iteration-bound
# / step / start argument itself is the wrong type.
#
# Probes (every form CPython rejects, mamba silently coerces):
#   • range(3.14)                       → mamba: []            (TypeError)
#   • range(None)                       → mamba: []            (TypeError)
#   • range('abc')                      → mamba: []            (TypeError)
#   • range([1,2])                      → mamba: []            (TypeError)
#   • range({1:2})                      → mamba: []            (TypeError)
#   • range((1,2))                      → mamba: []            (TypeError)
#   • range(0, 3.14)                    → mamba: []            (TypeError)
#   • range(0, None)                    → mamba: []            (TypeError)
#   • range(0, 'abc')                   → mamba: []            (TypeError)
#   • range(0, [1,2])                   → mamba: []            (TypeError)
#   • range(0, 5, 'a')                  → mamba: [0,1,2,3,4]   (TypeError)
#   • range(0, 5, None)                 → mamba: [0,1,2,3,4]   (TypeError)
#   • range(0, 5, 1.5)                  → mamba: [0,1,2,3,4]   (TypeError)
#   • range(0, 5, [1])                  → mamba: [0,1,2,3,4]   (TypeError)
#   • enumerate([1,2], 3.14)            → mamba: [(0,1),(1,2)] (TypeError)
#   • enumerate([1,2], None)            → mamba: [(0,1),(1,2)] (TypeError)
#   • enumerate([1,2], 'abc')           → mamba: [(0,1),(1,2)] (TypeError)
#   • enumerate([1,2], [1])             → mamba: [(0,1),(1,2)] (TypeError)
#   • enumerate([1,2], {1:2})           → mamba: [(0,1),(1,2)] (TypeError)
#   • enumerate([1,2], start=3.14)      → mamba: [(0,1),(1,2)] (TypeError)
#
# CPython contract (uniform across every form):
#   range(non_int) / range(_, non_int) / range(_, _, non_int)
#       → TypeError("'<type>' object cannot be interpreted as an
#                    integer");
#   enumerate(it, start=non_int)
#       → TypeError("'<type>' object cannot be interpreted as an
#                    integer").
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so
# the runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_f: Any = 3.14
_n: Any = None
_s: Any = 'abc'
_l: Any = [1, 2]
_d: Any = {1: 2}
_t: Any = (1, 2)

# range(3.14) — stop must be int
try:
    _ = list(range(_f))
    raise AssertionError("range(3.14) must raise TypeError")
except TypeError:
    _ledger.append(1)

# range(None)
try:
    _ = list(range(_n))
    raise AssertionError("range(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# range('abc')
try:
    _ = list(range(_s))
    raise AssertionError("range('abc') must raise TypeError")
except TypeError:
    _ledger.append(1)

# range([1,2])
try:
    _ = list(range(_l))
    raise AssertionError("range([1,2]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# range({1:2})
try:
    _ = list(range(_d))
    raise AssertionError("range({1:2}) must raise TypeError")
except TypeError:
    _ledger.append(1)

# range((1,2))
try:
    _ = list(range(_t))
    raise AssertionError("range((1,2)) must raise TypeError")
except TypeError:
    _ledger.append(1)

# range(0, 3.14) — stop must be int
try:
    _ = list(range(0, _f))
    raise AssertionError("range(0, 3.14) must raise TypeError")
except TypeError:
    _ledger.append(1)

# range(0, None)
try:
    _ = list(range(0, _n))
    raise AssertionError("range(0, None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# range(0, 'abc')
try:
    _ = list(range(0, _s))
    raise AssertionError("range(0, 'abc') must raise TypeError")
except TypeError:
    _ledger.append(1)

# range(0, [1,2])
try:
    _ = list(range(0, _l))
    raise AssertionError("range(0, [1,2]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# range(0, 5, 'a') — step must be int
try:
    _ = list(range(0, 5, _s))
    raise AssertionError("range(0, 5, 'a') must raise TypeError")
except TypeError:
    _ledger.append(1)

# range(0, 5, None)
try:
    _ = list(range(0, 5, _n))
    raise AssertionError("range(0, 5, None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# range(0, 5, 1.5)
try:
    _ = list(range(0, 5, _f))
    raise AssertionError("range(0, 5, 1.5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# range(0, 5, [1])
try:
    _ = list(range(0, 5, _l))
    raise AssertionError("range(0, 5, [1]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# enumerate([1,2], 3.14) — start must be int
try:
    _ = list(enumerate([1, 2], _f))
    raise AssertionError("enumerate([1,2], 3.14) must raise TypeError")
except TypeError:
    _ledger.append(1)

# enumerate([1,2], None)
try:
    _ = list(enumerate([1, 2], _n))
    raise AssertionError("enumerate([1,2], None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# enumerate([1,2], 'abc')
try:
    _ = list(enumerate([1, 2], _s))
    raise AssertionError("enumerate([1,2], 'abc') must raise TypeError")
except TypeError:
    _ledger.append(1)

# enumerate([1,2], [1])
try:
    _ = list(enumerate([1, 2], _l))
    raise AssertionError("enumerate([1,2], [1]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# enumerate([1,2], {1:2})
try:
    _ = list(enumerate([1, 2], _d))
    raise AssertionError("enumerate([1,2], {1:2}) must raise TypeError")
except TypeError:
    _ledger.append(1)

# enumerate([1,2], start=3.14) — keyword form
try:
    _ = list(enumerate([1, 2], start=_f))
    raise AssertionError("enumerate([1,2], start=3.14) must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_range_enumerate_arg_type_silent {sum(_ledger)} asserts")
