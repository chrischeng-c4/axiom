# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_textwrap_string_template_silent"
# subject = "cpython321.lang_textwrap_string_template_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_textwrap_string_template_silent.py"
# status = "filled"
# ///
"""cpython321.lang_textwrap_string_template_silent: execute CPython 3.12 seed lang_textwrap_string_template_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences across the
# textwrap wrapping contract + the string.printable constant +
# the string.Template constructor + the documented textwrap.
# TextWrapper / shlex.shlex class identifier surface pinned by
# atomic 176: `textwrap` (the documented `wrap` / `fill` /
# `shorten` width-aware wrapping contract + the documented
# `TextWrapper` class identifier surface), `string` (the
# documented `printable` constant value + module hasattr
# contract + the documented `Template(...)` constructor instance
# class identity + the documented `.substitute(...)` instance
# method surface), and `shlex` (the documented `shlex` class
# identifier surface).
#
# The matching subset (bisect full module-level helper layer
# + hasattr surface, heapq full module-level helper layer +
# hasattr surface, copy full shallow + deep + hasattr surface,
# string constant value contract (ascii_lowercase /
# ascii_uppercase / ascii_letters / digits / punctuation /
# whitespace / hexdigits / octdigits) + string.capwords +
# partial string hasattr surface, shlex.split / quote / join
# + partial shlex hasattr surface, textwrap.dedent / indent
# + partial textwrap hasattr surface) is covered by
# `test_bisect_heapq_copy_string_value_ops`; this fixture
# pins the CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • textwrap.wrap("hello world this is a test", width=10) ==
#     ["hello", "world this", "is a test"] — documented width-
#     aware wrapping (mamba: returns ["hello world this is a
#     test"] — the width= argument is ignored and the
#     documented wrapping contract is broken);
#   • textwrap.fill("hello world this is a test", width=10) ==
#     "hello\nworld this\nis a test" — documented width-aware
#     fill (mamba: returns the input string unchanged);
#   • textwrap.shorten("hello world this is a test", width=15)
#     == "hello [...]" — documented placeholder-truncation
#     (mamba: returns the input string unchanged);
#   • len(string.printable) == 100 — documented printable
#     constant length (mamba: 0 — the constant is the empty
#     string and is broken);
#   • hasattr(string, "printable") is True — documented module
#     attribute (mamba: False — printable is missing from the
#     module hasattr surface entirely);
#   • type(string.Template("$x")).__name__ == "Template" —
#     documented Template constructor class identity (mamba:
#     returns "dict" — the constructor produces a `dict`
#     instead of the documented Template instance);
#   • string.Template("Hello, $name!").substitute(name="World")
#     == "Hello, World!" — documented Template.substitute
#     instance method (mamba: AttributeError 'dict' object has
#     no attribute 'substitute');
#   • hasattr(textwrap, "TextWrapper") is True — documented
#     class identifier (mamba: False);
#   • hasattr(shlex, "shlex") is True — documented class
#     identifier (mamba: False).
import textwrap as _textwrap_mod
import string as _string_mod
import shlex as _shlex_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# module-level helpers / class identifiers / instance methods
# that mamba's bundled type stubs do not surface accurately.
textwrap: Any = _textwrap_mod
string: Any = _string_mod
shlex: Any = _shlex_mod


_ledger: list[int] = []

# 1) textwrap.wrap — width-aware wrapping
assert textwrap.wrap("hello world this is a test", width=10) == ["hello", "world this", "is a test"]; _ledger.append(1)

# 2) textwrap.fill — width-aware fill
assert textwrap.fill("hello world this is a test", width=10) == "hello\nworld this\nis a test"; _ledger.append(1)

# 3) textwrap.shorten — placeholder-truncation
assert textwrap.shorten("hello world this is a test", width=15) == "hello [...]"; _ledger.append(1)

# 4) string.printable — constant length + hasattr
assert len(string.printable) == 100; _ledger.append(1)
assert hasattr(string, "printable") == True; _ledger.append(1)

# 5) string.Template — constructor class identity
assert type(string.Template("$x")).__name__ == "Template"; _ledger.append(1)

# 6) string.Template.substitute — instance method
assert string.Template("Hello, $name!").substitute(name="World") == "Hello, World!"; _ledger.append(1)

# 7) textwrap.TextWrapper — class identifier hasattr
assert hasattr(textwrap, "TextWrapper") == True; _ledger.append(1)

# 8) shlex.shlex — class identifier hasattr
assert hasattr(shlex, "shlex") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_textwrap_string_template_silent {sum(_ledger)} asserts")
