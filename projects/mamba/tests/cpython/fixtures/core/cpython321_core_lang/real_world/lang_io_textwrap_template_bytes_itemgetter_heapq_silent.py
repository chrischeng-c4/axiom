# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_io_textwrap_template_bytes_itemgetter_heapq_silent"
# subject = "cpython321.lang_io_textwrap_template_bytes_itemgetter_heapq_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_io_textwrap_template_bytes_itemgetter_heapq_silent.py"
# status = "filled"
# ///
"""cpython321.lang_io_textwrap_template_bytes_itemgetter_heapq_silent: execute CPython 3.12 seed lang_io_textwrap_template_bytes_itemgetter_heapq_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the
# silent value-contract divergence of the
# `io` deep surface + StringIO/BytesIO write+getvalue value
# contract / `textwrap` TextWrapper + wrap/shorten value
# contract / `string.Template / string.Formatter` instance
# calls / `bytes.upper / bytes.lower` / `bytearray.__setitem__`
# / `operator.itemgetter` call result / `heapq.merge`
# reverse= / `statistics` float-returning ops boxed-handle int
# / `fractions.Fraction` arithmetic + string ctor + from_float
# ten-pack pinned to atomic 241: `io.TextIOWrapper / io.BufferedReader /
# io.BufferedWriter / io.FileIO / io.IOBase / io.RawIOBase /
# io.UnsupportedOperation / io.DEFAULT_BUFFER_SIZE` (the
# documented deep surface — mamba's `io` module dict only
# exposes `StringIO / BytesIO`), `io.StringIO().write(s) /
# io.BytesIO().write(b)` followed by `.getvalue()` (the
# documented "write persists to the in-memory buffer and
# getvalue returns what was written" value contract — mamba's
# StringIO/BytesIO silently drop every write and getvalue
# always returns the empty string/bytes), `textwrap.TextWrapper`
# (the documented top-level class — mamba does not expose it),
# `textwrap.wrap(s, width=10)` (the documented "split the
# string into lines no longer than width" value contract —
# mamba silently returns a single-element list with the full
# input regardless of width) and `textwrap.shorten(s, width=10)`
# (the documented "truncate the string to width and append
# placeholder" value contract — mamba silently returns the
# full input unmodified), `string.Template(...).substitute(...) /
# .safe_substitute(...) / string.Formatter().format(...)` (the
# documented instance-method surface — mamba's Template /
# Formatter instances are bare dicts that raise AttributeError
# at the documented call site), `bytearray(b"...").__setitem__(...)`
# (the documented "bytearray is mutable" value contract —
# mamba's bytearray silently rejects `__setitem__` with
# AttributeError), `b"hello".upper() / b"hello".lower()` (the
# documented byte-case methods — mamba's bytes object exposes
# no such attribute), `operator.itemgetter(...)(seq)` (the
# documented "itemgetter returns a callable that indexes its
# argument" value contract — mamba's itemgetter call silently
# returns None), `heapq.merge([7, 4, 1], [8, 5, 2], reverse=True)`
# (the documented "reverse= flag merges in descending order"
# value contract — mamba silently ignores reverse= and returns
# the ascending merge), `statistics.median_grouped /
# statistics.fmean / statistics.geometric_mean` (the documented
# "float-returning op returns a Python `float`" value contract —
# mamba silently returns a boxed-handle integer whose
# `type(...).__name__` resolves to `int` instead of `float`),
# and `fractions.Fraction(1, 2) + Fraction(1, 3) / str(Fraction(1, 2))
# / Fraction("3/4") / Fraction.from_float(0.5)` (the documented
# arithmetic + string-ctor + from_float surface — mamba's
# Fraction arithmetic silently returns a UUID-like opaque
# handle, `str(Fraction(1, 2))` renders as a giant integer
# string instead of `"1/2"`, `Fraction("3/4")` returns a giant
# integer instead of a Fraction, and `Fraction.from_float(0.5)`
# returns None).
#
# Behavioral edges that CONFORM on mamba (io.StringIO/BytesIO
# class binding; textwrap.wrap/fill/dedent/indent/shorten
# hasattr + dedent + indent value ops; string.Formatter/Template
# class binding; bytes literal + decode + hex + fromhex +
# bytearray ctor + concat + len + index + slice + in + split +
# join + startswith + endswith; operator 34-name hasattr +
# add/sub/mul/eq/ne/lt/gt/le/ge value ops; heapq.merge basic;
# statistics 8-name hasattr + multimode + quantiles;
# fractions.Fraction class binding) are covered in the matching
# pass fixture
# `test_io_textwrap_string_bytes_operator_heapq_statistics_value_ops`.
from typing import Any
import io as _io_mod
import textwrap as _textwrap_mod
import string as _string_mod
import operator as _operator_mod
import heapq as _heapq_mod
import statistics as _statistics_mod
import fractions as _fractions_mod

io_mod: Any = _io_mod
textwrap_mod: Any = _textwrap_mod
string_mod: Any = _string_mod
operator_mod: Any = _operator_mod
heapq_mod: Any = _heapq_mod
statistics_mod: Any = _statistics_mod
fractions_mod: Any = _fractions_mod


_ledger: list[int] = []

# 1) io deep surface
#    (mamba: missing — only StringIO/BytesIO are exposed)
assert hasattr(io_mod, "TextIOWrapper") == True; _ledger.append(1)
assert hasattr(io_mod, "BufferedReader") == True; _ledger.append(1)
assert hasattr(io_mod, "BufferedWriter") == True; _ledger.append(1)
assert hasattr(io_mod, "FileIO") == True; _ledger.append(1)
assert hasattr(io_mod, "IOBase") == True; _ledger.append(1)
assert hasattr(io_mod, "RawIOBase") == True; _ledger.append(1)
assert hasattr(io_mod, "UnsupportedOperation") == True; _ledger.append(1)
assert hasattr(io_mod, "DEFAULT_BUFFER_SIZE") == True; _ledger.append(1)

# 2) io.StringIO / io.BytesIO write+getvalue value contract
#    (mamba: silently drops writes — getvalue always returns '')
_s1 = io_mod.StringIO()
_s1.write("hello")
assert _s1.getvalue() == "hello"; _ledger.append(1)
_s2 = io_mod.BytesIO()
_s2.write(b"abc")
assert _s2.getvalue() == b"abc"; _ledger.append(1)

# 3) textwrap.TextWrapper — top-level class
#    (mamba: missing)
assert hasattr(textwrap_mod, "TextWrapper") == True; _ledger.append(1)

# 4) textwrap.wrap — width-bounded line split
#    (mamba: silently returns a single-element list with the full input)
assert textwrap_mod.wrap("hello world foo bar baz", width=10) == ["hello", "world foo", "bar baz"]; _ledger.append(1)

# 5) textwrap.shorten — truncate-to-width
#    (mamba: silently returns the full input unmodified)
assert textwrap_mod.shorten("hello world foo bar", width=10) == "[...]"; _ledger.append(1)

# 6) string.Template / string.Formatter instance calls
#    (mamba: Template/Formatter instances are bare dicts — AttributeError at call site)
assert string_mod.Template("$x + $y").substitute(x="a", y="b") == "a + b"; _ledger.append(1)
assert string_mod.Template("$x + $y").safe_substitute(x="a") == "a + $y"; _ledger.append(1)
assert string_mod.Formatter().format("{0} {1}", "hello", "world") == "hello world"; _ledger.append(1)

# 7) bytearray.__setitem__ — mutable byte assignment
#    (mamba: AttributeError — silently treats bytearray as immutable bytes)
_ba = bytearray(b"hello")
_ba[0] = ord("X")
assert bytes(_ba) == b"Xello"; _ledger.append(1)

# 8) bytes.upper / bytes.lower
#    (mamba: AttributeError — methods not exposed on bytes object)
assert b"hello".upper() == b"HELLO"; _ledger.append(1)
assert b"HELLO".lower() == b"hello"; _ledger.append(1)

# 9) operator.itemgetter call result
#    (mamba: itemgetter call silently returns None)
assert operator_mod.itemgetter(1)([10, 20, 30]) == 20; _ledger.append(1)
assert operator_mod.itemgetter(0, 2)([10, 20, 30]) == (10, 30); _ledger.append(1)

# 10) heapq.merge reverse=True
#     (mamba: reverse= silently ignored — ascending merge returned)
assert list(heapq_mod.merge([7, 4, 1], [8, 5, 2], reverse=True)) == [8, 7, 5, 4, 2, 1]; _ledger.append(1)

# 11) statistics float-returning ops — type contract
#     (mamba: returns boxed-handle int instead of float)
assert type(statistics_mod.median_grouped([1, 2, 2, 3, 4, 4, 4, 5])).__name__ == "float"; _ledger.append(1)
assert type(statistics_mod.fmean([1, 2, 3, 4])).__name__ == "float"; _ledger.append(1)
assert type(statistics_mod.geometric_mean([1, 4, 16])).__name__ == "float"; _ledger.append(1)

# 12) fractions.Fraction arithmetic + string ctor + from_float
#     (mamba: arithmetic returns UUID-like opaque handle, str renders giant int,
#     string ctor returns giant int, from_float returns None)
assert (fractions_mod.Fraction(1, 2) + fractions_mod.Fraction(1, 3)) == fractions_mod.Fraction(5, 6); _ledger.append(1)
assert str(fractions_mod.Fraction(1, 2)) == "1/2"; _ledger.append(1)
assert fractions_mod.Fraction("3/4") == fractions_mod.Fraction(3, 4); _ledger.append(1)
assert fractions_mod.Fraction.from_float(0.5) == fractions_mod.Fraction(1, 2); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_io_textwrap_template_bytes_itemgetter_heapq_silent {sum(_ledger)} asserts")
