# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_string_textwrap_ast_pprint_silent"
# subject = "cpython321.lang_string_textwrap_ast_pprint_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_string_textwrap_ast_pprint_silent.py"
# status = "filled"
# ///
"""cpython321.lang_string_textwrap_ast_pprint_silent: execute CPython 3.12 seed lang_string_textwrap_ast_pprint_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences across the
# text / parse / pretty-print quintet pinned by atomic 147:
# `mimetypes` (the MimeTypes class identity), `string` (the
# printable length contract, the Template instance .substitute
# method + Template / Formatter class identity, and the capwords
# helper), `textwrap` (the documented fill / wrap / dedent /
# shorten word-wrap helpers — all silently broken on mamba),
# `ast` (the parse mode="eval" returns Expression contract, the
# literal_eval list / tuple / dict surface, the AST / Module /
# Expression / BinOp class identity, and the dump representation),
# and `pprint` (the documented one-line layout for short
# containers + the PrettyPrinter / isreadable / isrecursive
# helpers).
#
# The matching subset (mimetypes.guess_type / guess_extension /
# inited, fnmatch.fnmatch / filter / translate / fnmatchcase,
# string.ascii_lowercase / ascii_uppercase / ascii_letters /
# digits / hexdigits / octdigits / punctuation) is covered by
# `test_mimetypes_fnmatch_string_value_ops`; this fixture pins
# the CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • mimetypes.MimeTypes.__name__ == "MimeTypes" — class
#     identity (mamba: returns None);
#   • len(string.printable) == 100 — ASCII printable + digits +
#     letters + punctuation + whitespace (mamba: returns 0 —
#     printable is the empty string);
#   • string.Template("$name").substitute(name="World") ==
#     "World" — PEP 292 string template (mamba: AttributeError,
#     Template instance has no .substitute);
#   • string.Template.__name__ == "Template" (mamba: None);
#   • string.Formatter.__name__ == "Formatter" — PEP 3101 string
#     formatter (mamba: None);
#   • string.capwords("hello world") == "Hello World" — word
#     capitalizer (mamba: AttributeError, string is a `dict`);
#   • textwrap.fill("hello world this is text", width=10)
#     wraps to "hello\nworld this\nis text" — width-bounded
#     word wrap (mamba: returns the input unchanged);
#   • textwrap.wrap("hello world", width=5) ==
#     ["hello", "world"] — split into width-bounded segments
#     (mamba: returns ["hello world"], no split);
#   • textwrap.dedent("    hello\n    world\n") ==
#     "hello\nworld\n" — preserves trailing newline (mamba:
#     drops the trailing newline);
#   • textwrap.shorten("hello world this is a test", width=15)
#     == "hello [...]" — width-bounded truncation (mamba:
#     returns the input unchanged);
#   • type(ast.parse("1+2", mode="eval")).__name__ ==
#     "Expression" — eval-mode parse returns Expression (mamba:
#     returns "Module" — mamba ignores the mode arg);
#   • ast.literal_eval("[1, 2, 3]") == [1, 2, 3] — list literal
#     (mamba: returns None);
#   • ast.literal_eval("(1, 2)") == (1, 2) — tuple literal
#     (mamba: returns None);
#   • ast.literal_eval("{'a': 1}") == {"a": 1} — dict literal
#     (mamba: returns None);
#   • ast.AST.__name__ == "AST" — abstract-base class identity
#     (mamba: None);
#   • ast.Module.__name__ == "Module" (mamba: None);
#   • ast.Expression.__name__ == "Expression" (mamba: None);
#   • ast.BinOp.__name__ == "BinOp" (mamba: None);
#   • ast.dump(ast.parse("1+2", mode="eval")) starts with
#     "Expression(body=" — structured AST repr (mamba: returns
#     "Module()", an empty tree);
#   • pprint.pformat([1, 2, 3]) == "[1, 2, 3]" — short
#     container on one line (mamba: pretty-prints to multi-line
#     "[\\n 1,\\n 2,\\n 3\\n]");
#   • pprint.pformat({"a": 1, "b": 2}) == "{'a': 1, 'b': 2}"
#     (mamba: multi-line);
#   • pprint.PrettyPrinter.__name__ == "PrettyPrinter" (mamba:
#     AttributeError, pprint is a `dict`);
#   • pprint.isreadable([1, 2, 3]) is True (mamba:
#     AttributeError).
import mimetypes as _mimetypes_mod
import string as _string_mod
import textwrap as _textwrap_mod
import ast as _ast_mod
import pprint as _pprint_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identifiers / module-level helpers / instance methods
# that mamba's bundled type stubs do not surface accurately.
mimetypes: Any = _mimetypes_mod
string: Any = _string_mod
textwrap: Any = _textwrap_mod
ast: Any = _ast_mod
pprint: Any = _pprint_mod

_ledger: list[int] = []

# 1) mimetypes.MimeTypes — class identity
assert mimetypes.MimeTypes.__name__ == "MimeTypes"; _ledger.append(1)

# 2) string.printable — ASCII printable-character ceiling
assert len(string.printable) == 100; _ledger.append(1)

# 3) string.Template — PEP 292 string template
_t = string.Template("$name")
assert _t.substitute(name="World") == "World"; _ledger.append(1)
assert string.Template.__name__ == "Template"; _ledger.append(1)

# 4) string.Formatter — PEP 3101 class identity
assert string.Formatter.__name__ == "Formatter"; _ledger.append(1)

# 5) string.capwords — word capitalizer
assert string.capwords("hello world") == "Hello World"; _ledger.append(1)

# 6) textwrap.fill — width-bounded word wrap
assert textwrap.fill("hello world this is text", width=10) == "hello\nworld this\nis text"; _ledger.append(1)

# 7) textwrap.wrap — split into width-bounded segments
assert textwrap.wrap("hello world", width=5) == ["hello", "world"]; _ledger.append(1)

# 8) textwrap.dedent — preserves trailing newline
assert textwrap.dedent("    hello\n    world\n") == "hello\nworld\n"; _ledger.append(1)

# 9) textwrap.shorten — width-bounded truncation
assert textwrap.shorten("hello world this is a test", width=15) == "hello [...]"; _ledger.append(1)

# 10) ast.parse — eval-mode returns Expression
assert type(ast.parse("1+2", mode="eval")).__name__ == "Expression"; _ledger.append(1)

# 11) ast.literal_eval — list / tuple / dict literals
assert ast.literal_eval("[1, 2, 3]") == [1, 2, 3]; _ledger.append(1)
assert ast.literal_eval("(1, 2)") == (1, 2); _ledger.append(1)
assert ast.literal_eval("{'a': 1}") == {"a": 1}; _ledger.append(1)

# 12) ast — class identity surface
assert ast.AST.__name__ == "AST"; _ledger.append(1)
assert ast.Module.__name__ == "Module"; _ledger.append(1)
assert ast.Expression.__name__ == "Expression"; _ledger.append(1)
assert ast.BinOp.__name__ == "BinOp"; _ledger.append(1)

# 13) ast.dump — structured AST repr
assert ast.dump(ast.parse("1+2", mode="eval")).startswith("Expression(body="); _ledger.append(1)

# 14) pprint.pformat — short-container one-line layout
assert pprint.pformat([1, 2, 3]) == "[1, 2, 3]"; _ledger.append(1)
assert pprint.pformat({"a": 1, "b": 2}) == "{'a': 1, 'b': 2}"; _ledger.append(1)

# 15) pprint.PrettyPrinter — class identity
assert pprint.PrettyPrinter.__name__ == "PrettyPrinter"; _ledger.append(1)

# 16) pprint.isreadable — readback-safety predicate
assert pprint.isreadable([1, 2, 3]) == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_string_textwrap_ast_pprint_silent {sum(_ledger)} asserts")
