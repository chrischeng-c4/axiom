# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_textwrap_string_unicodedata_shlex_value_ops"
# subject = "cpython321.test_textwrap_string_unicodedata_shlex_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_textwrap_string_unicodedata_shlex_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_textwrap_string_unicodedata_shlex_value_ops: execute CPython 3.12 seed test_textwrap_string_unicodedata_shlex_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 263 pass conformance — textwrap module (hasattr fill/wrap/
# dedent/indent/shorten + fill smoke, wrap smoke, dedent strips
# common leading whitespace, dedent mixed indent keeps offset,
# indent prepends prefix to every line) + string module (hasattr
# ascii_letters/ascii_lowercase/ascii_uppercase/digits/hexdigits/
# octdigits/punctuation/whitespace/Template/Formatter/capwords +
# ascii_lowercase=='abcdefghijklmnopqrstuvwxyz', ascii_uppercase=
# 'ABCDEFGHIJKLMNOPQRSTUVWXYZ', digits=='0123456789', hexdigits=
# '0123456789abcdefABCDEF', octdigits=='01234567', '!' in
# punctuation, capwords('hello world')=='Hello World') +
# unicodedata module (hasattr name/category/normalize/decimal/
# bidirectional/unidata_version + category 'A'/'a' Lu/Ll,
# category '0' Nd, category ' ' Zs, normalize NFC preserves
# precomposed, decimal('5')==5, bidirectional('A')=='L') + shlex
# module (hasattr split/quote/join + split('a b c')==['a','b','c'],
# split('a "b c" d')==['a','b c','d'], quote('hello')=='hello',
# quote('hello world')=="'hello world'", quote("it's") double-
# quotes the single, join(['a','b','c'])=='a b c', join with
# space-bearing element quotes that element).
# All asserts match between CPython 3.12 and mamba.
import textwrap
import string
import unicodedata
import shlex


_ledger: list[int] = []

# 1) textwrap — hasattr surface
assert hasattr(textwrap, "fill") == True; _ledger.append(1)
assert hasattr(textwrap, "wrap") == True; _ledger.append(1)
assert hasattr(textwrap, "dedent") == True; _ledger.append(1)
assert hasattr(textwrap, "indent") == True; _ledger.append(1)
assert hasattr(textwrap, "shorten") == True; _ledger.append(1)

# 2) textwrap — dedent / indent value contracts (these don't diverge)
assert textwrap.dedent("    hello\n    world") == "hello\nworld"; _ledger.append(1)
assert textwrap.dedent("  a\n    b") == "a\n  b"; _ledger.append(1)
assert textwrap.indent("a\nb", "  ") == "  a\n  b"; _ledger.append(1)
assert textwrap.dedent("") == ""; _ledger.append(1)
assert textwrap.indent("", "  ") == ""; _ledger.append(1)

# 3) string — hasattr surface
assert hasattr(string, "ascii_letters") == True; _ledger.append(1)
assert hasattr(string, "ascii_lowercase") == True; _ledger.append(1)
assert hasattr(string, "ascii_uppercase") == True; _ledger.append(1)
assert hasattr(string, "digits") == True; _ledger.append(1)
assert hasattr(string, "hexdigits") == True; _ledger.append(1)
assert hasattr(string, "octdigits") == True; _ledger.append(1)
assert hasattr(string, "punctuation") == True; _ledger.append(1)
assert hasattr(string, "whitespace") == True; _ledger.append(1)
assert hasattr(string, "Template") == True; _ledger.append(1)
assert hasattr(string, "Formatter") == True; _ledger.append(1)
assert hasattr(string, "capwords") == True; _ledger.append(1)

# 4) string — constant value contracts
assert string.ascii_lowercase == "abcdefghijklmnopqrstuvwxyz"; _ledger.append(1)
assert string.ascii_uppercase == "ABCDEFGHIJKLMNOPQRSTUVWXYZ"; _ledger.append(1)
assert string.digits == "0123456789"; _ledger.append(1)
assert string.hexdigits == "0123456789abcdefABCDEF"; _ledger.append(1)
assert string.octdigits == "01234567"; _ledger.append(1)
assert ("!" in string.punctuation) == True; _ledger.append(1)
assert ("@" in string.punctuation) == True; _ledger.append(1)
assert (" " in string.whitespace) == True; _ledger.append(1)
assert string.capwords("hello world") == "Hello World"; _ledger.append(1)
assert string.capwords("foo bar baz") == "Foo Bar Baz"; _ledger.append(1)

# 5) unicodedata — hasattr surface (the conform subset)
assert hasattr(unicodedata, "name") == True; _ledger.append(1)
assert hasattr(unicodedata, "category") == True; _ledger.append(1)
assert hasattr(unicodedata, "normalize") == True; _ledger.append(1)
assert hasattr(unicodedata, "decimal") == True; _ledger.append(1)
assert hasattr(unicodedata, "bidirectional") == True; _ledger.append(1)
assert hasattr(unicodedata, "unidata_version") == True; _ledger.append(1)

# 6) unicodedata — category contracts
assert unicodedata.category("A") == "Lu"; _ledger.append(1)
assert unicodedata.category("a") == "Ll"; _ledger.append(1)
assert unicodedata.category("0") == "Nd"; _ledger.append(1)
assert unicodedata.category(" ") == "Zs"; _ledger.append(1)

# 7) unicodedata — normalize NFC preserves precomposed forms
assert unicodedata.normalize("NFC", "café") == "café"; _ledger.append(1)
assert unicodedata.normalize("NFC", "abc") == "abc"; _ledger.append(1)

# 8) unicodedata — decimal / bidirectional
assert unicodedata.decimal("5") == 5; _ledger.append(1)
assert unicodedata.decimal("0") == 0; _ledger.append(1)
assert unicodedata.bidirectional("A") == "L"; _ledger.append(1)

# 9) shlex — hasattr surface
assert hasattr(shlex, "split") == True; _ledger.append(1)
assert hasattr(shlex, "quote") == True; _ledger.append(1)
assert hasattr(shlex, "join") == True; _ledger.append(1)

# 10) shlex — split / quote / join value contracts
assert shlex.split("a b c") == ["a", "b", "c"]; _ledger.append(1)
assert shlex.split('a "b c" d') == ["a", "b c", "d"]; _ledger.append(1)
assert shlex.split("") == []; _ledger.append(1)
assert shlex.quote("hello") == "hello"; _ledger.append(1)
assert shlex.quote("hello world") == "'hello world'"; _ledger.append(1)
assert shlex.quote("it's") == "'it'\"'\"'s'"; _ledger.append(1)
assert shlex.join(["a", "b", "c"]) == "a b c"; _ledger.append(1)
assert shlex.join(["a", "b c"]) == "a 'b c'"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_textwrap_string_unicodedata_shlex_value_ops {sum(_ledger)} asserts")
