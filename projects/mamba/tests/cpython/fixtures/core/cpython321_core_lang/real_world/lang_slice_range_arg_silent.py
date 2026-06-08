# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_slice_range_arg_silent"
# subject = "cpython321.lang_slice_range_arg_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_slice_range_arg_silent.py"
# status = "filled"
# ///
"""cpython321.lang_slice_range_arg_silent: execute CPython 3.12 seed lang_slice_range_arg_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Spec seed for CPython ValueError / TypeError contract on the
# slice / range argument-validation corners that mamba silently
# coerces to a None / empty-list result. Surface: CPython rejects
# (1) slicing any sequence with a zero step (`seq[::0]`) because the
# step is the iteration delta and 0 would loop forever — ValueError,
# not silent None; (2) `range(float)` / `range(str)` / `range(None)`
# because `range` requires int-interpretable arguments only — TypeError,
# not silent empty range; (3) `chr(negative_int)` because the
# Unicode codepoint domain is `[0, 0x10FFFF]` — ValueError, not silent
# None.
#
# Probes (every form CPython rejects, mamba silently coerces):
#   • [1,2,3][::0]                  → mamba: None      (ValueError)
#   • "abc"[::0]                    → mamba: None      (ValueError)
#   • (1,2,3)[::0]                  → mamba: None      (ValueError)
#   • b"abc"[::0]                   → mamba: None      (ValueError)
#   • range(5.5)                    → mamba: []        (TypeError)
#   • range("5")                    → mamba: []        (TypeError)
#   • range(None)                   → mamba: []        (TypeError)
#   • range(1, 5.5)                 → mamba: []        (TypeError)
#   • chr(-1)                       → mamba: None      (ValueError)
#   • chr(-100)                     → mamba: None      (ValueError)
#
# CPython contract:
#   seq[::0]               → ValueError("slice step cannot be zero");
#   range(non_int)         → TypeError("'<typename>' object cannot be
#                                  interpreted as an integer");
#   chr(negative_or_too_large)
#                          → ValueError("chr() arg not in
#                                  range(0x110000)").
#
# Note: existing `lang_index_codec_chr_silent.py` covers the
# upper-end `chr(0x110000)` ValueError and `ord('')` / `ord('ab')`
# TypeErrors — this seed adds the negative-codepoint and slice-step /
# range-non-int corners that fixture doesn't probe.
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so the
# runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_lst: Any = [1, 2, 3]
_str: Any = "abc"
_tup: Any = (1, 2, 3)
_b: Any = b"abc"
_step0: Any = slice(None, None, 0)
_f55: Any = 5.5
_str_5: Any = "5"
_n: Any = None
_neg1: Any = -1
_neg100: Any = -100

# list[::0] — slice step zero
try:
    _ = _lst[_step0]
    raise AssertionError("list[::0] must raise ValueError")
except ValueError:
    _ledger.append(1)

# "abc"[::0] — slice step zero on str
try:
    _ = _str[_step0]
    raise AssertionError("str[::0] must raise ValueError")
except ValueError:
    _ledger.append(1)

# tuple[::0] — slice step zero on tuple
try:
    _ = _tup[_step0]
    raise AssertionError("tuple[::0] must raise ValueError")
except ValueError:
    _ledger.append(1)

# b"abc"[::0] — slice step zero on bytes
try:
    _ = _b[_step0]
    raise AssertionError("bytes[::0] must raise ValueError")
except ValueError:
    _ledger.append(1)

# range(5.5) — float is not int-interpretable
try:
    _ = list(range(_f55))
    raise AssertionError("range(5.5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# range("5") — str is not int-interpretable
try:
    _ = list(range(_str_5))
    raise AssertionError("range('5') must raise TypeError")
except TypeError:
    _ledger.append(1)

# range(None) — None is not int-interpretable
try:
    _ = list(range(_n))
    raise AssertionError("range(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# range(1, 5.5) — second arg also rejected if non-int
try:
    _ = list(range(1, _f55))
    raise AssertionError("range(1, 5.5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# chr(-1) — negative codepoint is out of [0, 0x10FFFF]
try:
    _ = chr(_neg1)
    raise AssertionError("chr(-1) must raise ValueError")
except ValueError:
    _ledger.append(1)

# chr(-100) — same rule, further into the negative
try:
    _ = chr(_neg100)
    raise AssertionError("chr(-100) must raise ValueError")
except ValueError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_slice_range_arg_silent {sum(_ledger)} asserts")
