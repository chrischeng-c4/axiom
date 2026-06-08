# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_format_complex_fromhex_silent"
# subject = "cpython321.lang_format_complex_fromhex_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_format_complex_fromhex_silent.py"
# status = "filled"
# ///
"""cpython321.lang_format_complex_fromhex_silent: execute CPython 3.12 seed lang_format_complex_fromhex_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Spec seed for CPython ValueError / TypeError / KeyError / IndexError
# contract on the str.format / complex() / float.fromhex /
# bytes.fromhex / bytearray.fromhex / dict.update / set.update /
# list-indexing corners that mamba silently coerces. Surface:
# CPython rejects (1) `dict.update(<non-iterable-of-pairs>)` because
# dict update needs a mapping or iterable of key/value pairs —
# ValueError on a single-char-string element, TypeError on a non-
# iterable scalar, not silent `None`; (2) `set.update(int)` because
# set.update demands an iterable — TypeError, not silent `None`; (3)
# `float.fromhex(non_hex)` / `bytes.fromhex(non_hex)` /
# `bytearray.fromhex(non_hex)` because the input is not a valid
# hexadecimal string — ValueError, not silent empty `b''` /
# `bytearray(b'')`; (4) `"{x}".format()` because the named field has
# no matching keyword arg — KeyError, not silent `'{x}'`; (5)
# `"{0}".format()` / `"{0}-{1}".format("a")` because the positional
# index is out of range — IndexError, not silent empty / truncated
# substitution; (6) `complex(non_complex_str)` because the string
# does not parse as a complex literal — ValueError, not silent
# `None`; (7) `list[str]` because list indexing requires int or
# slice — TypeError, not silent `None`.
#
# Probes (every form CPython rejects, mamba silently coerces):
#   • {}.update("ab")              → mamba: None         (ValueError)
#   • {}.update(5)                 → mamba: None         (TypeError)
#   • {1,2,3}.update(5)            → mamba: None         (TypeError)
#   • float.fromhex("not-a-hex")   → mamba: b''          (ValueError)
#   • float.fromhex("")            → mamba: b''          (ValueError)
#   • "{x}".format()               → mamba: '{x}'        (KeyError)
#   • "{0}".format()               → mamba: ''           (IndexError)
#   • "{0}-{1}".format("a")        → mamba: 'a-'         (IndexError)
#   • complex("not-a-complex")     → mamba: None         (ValueError)
#   • complex("(1+2j")             → mamba: None         (ValueError)
#   • [1,2,3]["1"]                 → mamba: None         (TypeError)
#   • bytearray.fromhex("zz")      → mamba: bytearray(b'') (ValueError)
#   • bytes.fromhex("not-hex")     → mamba: b''          (ValueError)
#
# CPython contract:
#   dict.update(str-of-len!=2-pair)
#                       → ValueError("dictionary update sequence
#                              element #N has length L; 2 is required");
#   dict.update(int) / set.update(int)
#                       → TypeError("'int' object is not iterable");
#   float.fromhex(bad)  → ValueError("invalid hexadecimal floating-
#                              point string");
#   bytes.fromhex(bad)
#   bytearray.fromhex(bad)
#                       → ValueError("non-hexadecimal number found in
#                              fromhex() arg at position N");
#   str.format({named}) → KeyError(name);
#   str.format({i}) i out of range
#                       → IndexError("Replacement index N out of
#                              range for positional args tuple");
#   complex(bad_str)    → ValueError("complex() arg is a malformed
#                              string");
#   list[str]           → TypeError("list indices must be integers or
#                              slices, not str").
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so the
# runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_d1: Any = {}
_d2: Any = {}
_s: Any = {1, 2, 3}
_l: Any = [1, 2, 3]
_bad_hex: Any = "not-a-hex"
_empty_hex: Any = ""
_bad_bhex: Any = "not-hex"
_bad_bahex: Any = "zz"
_bad_complex1: Any = "not-a-complex"
_bad_complex2: Any = "(1+2j"
_bad_idx: Any = "1"

# dict.update("ab") — each elem is a 1-char string, not a 2-pair
try:
    _ = _d1.update("ab")
    raise AssertionError("{}.update('ab') must raise ValueError")
except ValueError:
    _ledger.append(1)

# dict.update(int) — int not iterable
try:
    _ = _d2.update(5)
    raise AssertionError("{}.update(5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# set.update(int) — int not iterable
try:
    _ = _s.update(5)
    raise AssertionError("set.update(5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# float.fromhex(non_hex)
try:
    _ = float.fromhex(_bad_hex)
    raise AssertionError("float.fromhex('not-a-hex') must raise ValueError")
except ValueError:
    _ledger.append(1)

# float.fromhex("")
try:
    _ = float.fromhex(_empty_hex)
    raise AssertionError("float.fromhex('') must raise ValueError")
except ValueError:
    _ledger.append(1)

# "{x}".format() — named field with no kw arg
try:
    _ = "{x}".format()
    raise AssertionError("'{x}'.format() must raise KeyError")
except KeyError:
    _ledger.append(1)

# "{0}".format() — positional index out of empty args
try:
    _ = "{0}".format()
    raise AssertionError("'{0}'.format() must raise IndexError")
except IndexError:
    _ledger.append(1)

# "{0}-{1}".format("a") — second index out of range
try:
    _ = "{0}-{1}".format("a")
    raise AssertionError("'{0}-{1}'.format('a') must raise IndexError")
except IndexError:
    _ledger.append(1)

# complex(non_complex_str)
try:
    _ = complex(_bad_complex1)
    raise AssertionError("complex('not-a-complex') must raise ValueError")
except ValueError:
    _ledger.append(1)

# complex("(1+2j") — unbalanced parens
try:
    _ = complex(_bad_complex2)
    raise AssertionError("complex('(1+2j') must raise ValueError")
except ValueError:
    _ledger.append(1)

# list[str] — indices must be int or slice
try:
    _ = _l[_bad_idx]
    raise AssertionError("[1,2,3]['1'] must raise TypeError")
except TypeError:
    _ledger.append(1)

# bytearray.fromhex("zz") — non-hex char
try:
    _ = bytearray.fromhex(_bad_bahex)
    raise AssertionError("bytearray.fromhex('zz') must raise ValueError")
except ValueError:
    _ledger.append(1)

# bytes.fromhex("not-hex")
try:
    _ = bytes.fromhex(_bad_bhex)
    raise AssertionError("bytes.fromhex('not-hex') must raise ValueError")
except ValueError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_format_complex_fromhex_silent {sum(_ledger)} asserts")
