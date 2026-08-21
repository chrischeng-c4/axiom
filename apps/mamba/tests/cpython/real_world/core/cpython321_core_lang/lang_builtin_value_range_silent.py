# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_builtin_value_range_silent"
# subject = "cpython321.lang_builtin_value_range_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_builtin_value_range_silent.py"
# status = "filled"
# ///
"""cpython321.lang_builtin_value_range_silent: execute CPython 3.12 seed lang_builtin_value_range_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Spec seed for CPython ValueError / TypeError contract on the
# builtin constructor / conversion corners that mamba silently
# accepts. Surface: CPython rejects (1) `bytes(-n)` / `bytearray(-n)`
# because byte-array size cannot be negative — ValueError("negative
# count"), not silent empty bytes / bytearray; (2) `range(_, _, 0)`
# because zero-step makes the range never advance —
# ValueError("range() arg 3 must not be zero"), not silent empty
# range; (3) `chr(-1)` / `chr(0x110000)` because the argument must
# be in `[0, 0x10FFFF]` (the Unicode codepoint range) —
# ValueError("chr() arg not in range(0x110000)"), not silent `None`;
# (4) `ord("")` / `ord("ab")` because `ord` needs exactly one
# character — TypeError("ord() expected a character, but string of
# length N found"), not silent `None`. Existing `lang_*_silent`
# seeds touch coercion / coerce-to-None corners but the builtin-
# constructor argument-range family hasn't been pinned yet.
#
# Probes (every form CPython rejects, mamba silently coerces):
#   • bytes(-5)             → mamba: b''             (ValueError)
#   • bytes(-1)             → mamba: b''             (ValueError)
#   • bytearray(-5)         → mamba: bytearray(b'')  (ValueError)
#   • bytearray(-1)         → mamba: bytearray(b'')  (ValueError)
#   • range(0, 10, 0)       → mamba: <range>         (ValueError)
#   • range(0, 0, 0)        → mamba: <range>         (ValueError)
#   • chr(-1)               → mamba: None            (ValueError)
#   • chr(0x110000)         → mamba: None            (ValueError)
#   • chr(0x200000)         → mamba: None            (ValueError)
#   • ord("")               → mamba: None            (TypeError)
#   • ord("ab")             → mamba: None            (TypeError)
#   • ord("abc")            → mamba: None            (TypeError)
#
# CPython contract:
#   bytes(-n) / bytearray(-n)
#       → ValueError("negative count");
#   range(_, _, 0)
#       → ValueError("range() arg 3 must not be zero");
#   chr(c) where c < 0 or c > 0x10FFFF
#       → ValueError("chr() arg not in range(0x110000)");
#   ord(s) where len(s) != 1
#       → TypeError("ord() expected a character, but string of
#                   length N found").
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so
# the runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_neg_five: Any = -5
_neg_one: Any = -1
_zero: Any = 0
_too_big: Any = 0x110000
_way_too_big: Any = 0x200000
_empty_str: Any = ""
_two_chars: Any = "ab"
_three_chars: Any = "abc"

# bytes(-5) — negative count
try:
    _ = bytes(_neg_five)
    raise AssertionError("bytes(-5) must raise ValueError")
except ValueError:
    _ledger.append(1)

# bytes(-1) — negative count
try:
    _ = bytes(_neg_one)
    raise AssertionError("bytes(-1) must raise ValueError")
except ValueError:
    _ledger.append(1)

# bytearray(-5) — negative count
try:
    _ = bytearray(_neg_five)
    raise AssertionError("bytearray(-5) must raise ValueError")
except ValueError:
    _ledger.append(1)

# bytearray(-1) — negative count
try:
    _ = bytearray(_neg_one)
    raise AssertionError("bytearray(-1) must raise ValueError")
except ValueError:
    _ledger.append(1)

# range(0, 10, 0) — zero step
try:
    _ = range(0, 10, _zero)
    raise AssertionError("range(0, 10, 0) must raise ValueError")
except ValueError:
    _ledger.append(1)

# range(0, 0, 0) — zero step regardless of start/stop
try:
    _ = range(0, 0, _zero)
    raise AssertionError("range(0, 0, 0) must raise ValueError")
except ValueError:
    _ledger.append(1)

# chr(-1) — below Unicode codepoint range
try:
    _ = chr(_neg_one)
    raise AssertionError("chr(-1) must raise ValueError")
except ValueError:
    _ledger.append(1)

# chr(0x110000) — above Unicode codepoint range
try:
    _ = chr(_too_big)
    raise AssertionError("chr(0x110000) must raise ValueError")
except ValueError:
    _ledger.append(1)

# chr(0x200000) — way above Unicode codepoint range
try:
    _ = chr(_way_too_big)
    raise AssertionError("chr(0x200000) must raise ValueError")
except ValueError:
    _ledger.append(1)

# ord("") — needs exactly one character
try:
    _ = ord(_empty_str)
    raise AssertionError("ord('') must raise TypeError")
except TypeError:
    _ledger.append(1)

# ord("ab") — two-char string
try:
    _ = ord(_two_chars)
    raise AssertionError("ord('ab') must raise TypeError")
except TypeError:
    _ledger.append(1)

# ord("abc") — three-char string
try:
    _ = ord(_three_chars)
    raise AssertionError("ord('abc') must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_builtin_value_range_silent {sum(_ledger)} asserts")
