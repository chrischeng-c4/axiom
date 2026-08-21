# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_string_textwrap_constants_dedent_ops"
# subject = "cpython321.test_string_textwrap_constants_dedent_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_string_textwrap_constants_dedent_ops.py"
# status = "filled"
# ///
"""cpython321.test_string_textwrap_constants_dedent_ops: execute CPython 3.12 seed test_string_textwrap_constants_dedent_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of `string`
# (the documented per-character-class string constants used by every
# tokenizer / scrubber) and `textwrap` (the dedent / indent / capwords
# helpers used by every CLI help formatter and inline-docstring
# generator). No fixture coverage yet for either.
#
# The matching subset between mamba and CPython is the constant-and-
# pure-transform layer: every per-character-class string constant has
# the documented characters (ascii_lowercase, ascii_uppercase, digits,
# hexdigits, octdigits, punctuation, whitespace), `string.capwords`
# title-cases word-separated input, `textwrap.dedent` strips common
# leading whitespace, and `textwrap.indent` prepends a prefix to every
# non-empty line.
#
# Surface in this fixture:
#   • string.ascii_lowercase == "abcdefghijklmnopqrstuvwxyz";
#   • string.ascii_uppercase == "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
#   • string.ascii_letters   == ascii_lowercase + ascii_uppercase;
#   • string.digits          == "0123456789";
#   • string.hexdigits       == "0123456789abcdefABCDEF";
#   • string.octdigits       == "01234567";
#   • string.punctuation     == 32-character ASCII punctuation set
#     (CPython sentinel: `!"#$%&\'()*+,-./:;<=>?@[\\]^_`{|}~`);
#   • string.whitespace      == " \\t\\n\\r\\x0b\\x0c" (6 chars);
#   • len() of each constant — 26 / 26 / 52 / 10 / 22 / 8 / 32 / 6;
#   • string.capwords("hello world") == "Hello World";
#   • string.capwords("  alpha beta gamma  ") == "Alpha Beta Gamma";
#   • textwrap.dedent("  abc\\n  def")    == "abc\\ndef";
#   • textwrap.dedent("\\tabc\\n\\tdef")  == "abc\\ndef";
#   • textwrap.indent("a\\nb", "> ")        == "> a\\n> b";
#   • textwrap.indent on multi-line input prepends the prefix to every
#     non-empty line ("alpha\\nbeta\\ngamma" -> ">>> alpha\\n>>> beta
#     \\n>>> gamma");
#   • module-level callable / hasattr surface for both.
#
# Behavioral edges that DIVERGE on mamba (string.printable being
# documented as the full superset, string.Template / Formatter class
# identity + instance methods, textwrap.fill / wrap actually wrapping,
# textwrap.shorten producing "[...]", textwrap.TextWrapper class
# identity, io.StringIO / BytesIO class identity + lifecycle, io.SEEK_*
# integer constants, argparse.ArgumentParser class identity + parse,
# argparse.SUPPRESS / OPTIONAL / REMAINDER / ZERO_OR_MORE / ONE_OR_MORE
# sentinels) are covered in
# `lang_string_template_textwrap_wrap_io_argparse_silent.py`.
import string
import textwrap

_ledger: list[int] = []

# 1) Per-character-class constants
assert string.ascii_lowercase == "abcdefghijklmnopqrstuvwxyz"; _ledger.append(1)
assert string.ascii_uppercase == "ABCDEFGHIJKLMNOPQRSTUVWXYZ"; _ledger.append(1)
assert string.ascii_letters == string.ascii_lowercase + string.ascii_uppercase; _ledger.append(1)
assert string.digits == "0123456789"; _ledger.append(1)
assert string.hexdigits == "0123456789abcdefABCDEF"; _ledger.append(1)
assert string.octdigits == "01234567"; _ledger.append(1)
assert string.punctuation == "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~"; _ledger.append(1)
assert string.whitespace == " \t\n\r\x0b\x0c"; _ledger.append(1)

# 2) Constant lengths
assert len(string.ascii_lowercase) == 26; _ledger.append(1)
assert len(string.ascii_uppercase) == 26; _ledger.append(1)
assert len(string.ascii_letters) == 52; _ledger.append(1)
assert len(string.digits) == 10; _ledger.append(1)
assert len(string.hexdigits) == 22; _ledger.append(1)
assert len(string.octdigits) == 8; _ledger.append(1)
assert len(string.punctuation) == 32; _ledger.append(1)
assert len(string.whitespace) == 6; _ledger.append(1)

# 3) Type and overlap relationships
assert isinstance(string.ascii_lowercase, str); _ledger.append(1)
assert isinstance(string.digits, str); _ledger.append(1)
assert isinstance(string.punctuation, str); _ledger.append(1)
assert "0" in string.digits; _ledger.append(1)
assert "9" in string.digits; _ledger.append(1)
assert "f" in string.hexdigits; _ledger.append(1)
assert "F" in string.hexdigits; _ledger.append(1)
assert "8" not in string.octdigits; _ledger.append(1)
assert "a" in string.ascii_lowercase; _ledger.append(1)
assert "A" in string.ascii_uppercase; _ledger.append(1)
assert " " in string.whitespace; _ledger.append(1)
assert "\t" in string.whitespace; _ledger.append(1)
assert "\n" in string.whitespace; _ledger.append(1)

# 4) string.capwords — word-separated title case
assert string.capwords("hello world") == "Hello World"; _ledger.append(1)
assert string.capwords("alpha beta gamma") == "Alpha Beta Gamma"; _ledger.append(1)
assert string.capwords("HELLO WORLD") == "Hello World"; _ledger.append(1)
assert callable(string.capwords); _ledger.append(1)

# 5) textwrap.dedent — strips common leading whitespace
assert textwrap.dedent("  abc\n  def") == "abc\ndef"; _ledger.append(1)
assert textwrap.dedent("\tabc\n\tdef") == "abc\ndef"; _ledger.append(1)
assert textwrap.dedent("abc\ndef") == "abc\ndef"; _ledger.append(1)
assert textwrap.dedent("    abc\n  def") == "  abc\ndef"; _ledger.append(1)
assert callable(textwrap.dedent); _ledger.append(1)

# 6) textwrap.indent — prepend prefix to non-empty lines
assert textwrap.indent("a\nb", "> ") == "> a\n> b"; _ledger.append(1)
assert textwrap.indent("alpha\nbeta\ngamma", ">>> ") == ">>> alpha\n>>> beta\n>>> gamma"; _ledger.append(1)
assert textwrap.indent("", "> ") == ""; _ledger.append(1)
assert callable(textwrap.indent); _ledger.append(1)

# 7) hasattr — every documented constant + callable is exposed
assert hasattr(string, "ascii_lowercase"); _ledger.append(1)
assert hasattr(string, "ascii_uppercase"); _ledger.append(1)
assert hasattr(string, "ascii_letters"); _ledger.append(1)
assert hasattr(string, "digits"); _ledger.append(1)
assert hasattr(string, "hexdigits"); _ledger.append(1)
assert hasattr(string, "octdigits"); _ledger.append(1)
assert hasattr(string, "punctuation"); _ledger.append(1)
assert hasattr(string, "whitespace"); _ledger.append(1)
assert hasattr(string, "capwords"); _ledger.append(1)
assert hasattr(textwrap, "dedent"); _ledger.append(1)
assert hasattr(textwrap, "indent"); _ledger.append(1)

# NB: string.printable being the documented superset, string.Template
# and string.Formatter class identity + instance methods, textwrap.
# fill / wrap actually wrapping at the requested width, textwrap.
# shorten producing the documented "[...]" sentinel, textwrap.
# TextWrapper class identity, io.StringIO / BytesIO class identity +
# lifecycle, io.SEEK_* integer constants, argparse.ArgumentParser
# class identity + parse_args, argparse.SUPPRESS / OPTIONAL / REMAINDER
# / ZERO_OR_MORE / ONE_OR_MORE sentinels all DIVERGE on mamba — moved
# to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_string_textwrap_constants_dedent_ops {sum(_ledger)} asserts")
