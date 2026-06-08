# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_format_attr_codec_silent"
# subject = "cpython321.lang_format_attr_codec_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_format_attr_codec_silent.py"
# status = "filled"
# ///
"""cpython321.lang_format_attr_codec_silent: execute CPython 3.12 seed lang_format_attr_codec_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Spec seed for CPython contract on format-code-mismatch (ValueError),
# non-str attribute names (TypeError), call-of-non-callable
# (TypeError), unhashable-hash (TypeError), and unknown-codec
# (LookupError) / invalid-utf8 (UnicodeDecodeError). Surface:
# CPython rejects every form below with the indicated subclass of
# Exception; mamba 0.3.60 silently coerces / returns `0` / `None` /
# the original input instead of dispatching the protocol fallback.
# Existing lang_typeerror_* / lang_overflowerror_* / lang_valueerror_*
# seeds cover binary arithmetic / call-arity / iter-required /
# numeric-conversion / bitwise-unary / seq-repeat-builtin /
# str-bytes-method-arg angles; this seed adds the
# format-code-mismatch, attr-name-not-str, call-non-callable,
# unhashable-hash, unknown-codec, and invalid-utf8 corners.
#
# Probes (CPython raises the indicated exception; mamba silently
# returns a wrong-shape value):
#   • format('abc', 'd')           → mamba: '0'    (CPython: ValueError)
#   • format(3.5, 'd')             → mamba: '3'    (CPython: ValueError)
#   • '{:d}'.format('abc')         → mamba: '0'    (CPython: ValueError)
#   • getattr(o, 1)                → mamba: None   (CPython: TypeError)
#   • hasattr(o, 1)                → mamba: False  (CPython: TypeError)
#   • setattr(o, 1, 'v')           → mamba: silent (CPython: TypeError)
#   • 5()                          → mamba: None   (CPython: TypeError)
#   • hash({})                     → mamba: int    (CPython: TypeError)
#   • 'abc'.encode('not-a-codec')  → mamba: b'abc' (CPython: LookupError)
#   • b'abc'.decode('not-a-codec') → mamba: 'abc'  (CPython: LookupError)
#   • b'\xff\xff'.decode('utf-8')  → mamba: 'replaced' (CPython: UnicodeDecodeError)
#
# CPython contract:
#   format(str, 'd') / '{:d}'.format(str)
#                             → ValueError("Unknown format code 'd'
#                                  for object of type 'str'");
#   format(float, 'd')        → ValueError("Unknown format code 'd'
#                                  for object of type 'float'");
#   getattr(o, non_str)       → TypeError("attribute name must be
#                                  string, not '<typename>'");
#   hasattr(o, non_str)       → TypeError(same);
#   setattr(o, non_str, _)    → TypeError(same);
#   non_callable()            → TypeError("'<typename>' object is
#                                  not callable");
#   hash(unhashable)          → TypeError("unhashable type:
#                                  '<typename>'");
#   str.encode(bad_codec)     → LookupError("unknown encoding:
#                                  <name>");
#   bytes.decode(bad_codec)   → LookupError(same);
#   bytes.decode(invalid_utf8)→ UnicodeDecodeError("'utf-8' codec
#                                  can't decode byte ...").
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so the
# runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []


class _O:
    pass


_o = _O()
_int_name: Any = 1
_int_val: Any = 5
_dict_val: Any = {}
_abc: Any = "abc"
_flt: Any = 3.5
_fmt_d: Any = "{:d}"

# format('abc', 'd') — format code 'd' on str
try:
    _ = format(_abc, "d")
    raise AssertionError("format('abc', 'd') must raise ValueError")
except ValueError:
    _ledger.append(1)

# format(3.5, 'd') — format code 'd' on float
try:
    _ = format(_flt, "d")
    raise AssertionError("format(3.5, 'd') must raise ValueError")
except ValueError:
    _ledger.append(1)

# '{:d}'.format('abc') — same divergence via str.format(...)
try:
    _ = _fmt_d.format(_abc)
    raise AssertionError("'{:d}'.format('abc') must raise ValueError")
except ValueError:
    _ledger.append(1)

# getattr(o, 1) — attribute name must be string
try:
    _ = getattr(_o, _int_name)
    raise AssertionError("getattr(o, 1) must raise TypeError")
except TypeError:
    _ledger.append(1)

# hasattr(o, 1) — same non-str attr-name rejection
try:
    _ = hasattr(_o, _int_name)
    raise AssertionError("hasattr(o, 1) must raise TypeError")
except TypeError:
    _ledger.append(1)

# setattr(o, 1, 'v') — same non-str attr-name rejection
try:
    setattr(_o, _int_name, "v")
    raise AssertionError("setattr(o, 1, 'v') must raise TypeError")
except TypeError:
    _ledger.append(1)

# 5() — int is not callable
try:
    _ = _int_val()
    raise AssertionError("5() must raise TypeError")
except TypeError:
    _ledger.append(1)

# hash({}) — dict is unhashable
try:
    _ = hash(_dict_val)
    raise AssertionError("hash({}) must raise TypeError")
except TypeError:
    _ledger.append(1)

# 'abc'.encode('not-a-codec') — unknown encoding
try:
    _ = "abc".encode("not-a-codec")
    raise AssertionError("'abc'.encode('not-a-codec') must raise LookupError")
except LookupError:
    _ledger.append(1)

# b'abc'.decode('not-a-codec') — unknown encoding
try:
    _ = b"abc".decode("not-a-codec")
    raise AssertionError("b'abc'.decode('not-a-codec') must raise LookupError")
except LookupError:
    _ledger.append(1)

# b'\xff\xff'.decode('utf-8') — invalid UTF-8 byte sequence
try:
    _ = b"\xff\xff".decode("utf-8")
    raise AssertionError("b'\\xff\\xff'.decode('utf-8') must raise UnicodeDecodeError")
except UnicodeDecodeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_format_attr_codec_silent {sum(_ledger)} asserts")
