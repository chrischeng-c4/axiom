# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_re_string_textwrap_keyword_value_ops"
# subject = "cpython321.test_re_string_textwrap_keyword_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_re_string_textwrap_keyword_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_re_string_textwrap_keyword_value_ops: execute CPython 3.12 seed test_re_string_textwrap_keyword_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of the
# `re` / `string` / `textwrap` / `keyword` four-pack pinned to
# atomic 190: `re` (the documented full module-level helper
# hasattr surface — `compile` / `match` / `search` / `findall`
# / `finditer` / `sub` / `split` / `Pattern` / `Match` /
# `IGNORECASE` / `MULTILINE` / `DOTALL` / `VERBOSE` / `UNICODE`
# / `ASCII` / `escape` / `fullmatch` / `error` + the documented
# re.match group(0) / group(1) / group(2) capture contract +
# the documented re.search group contract + the documented
# re.findall / re.sub / re.split value contract + the
# documented re.escape value contract + the documented
# re.IGNORECASE == 2 integer-equivalence sentinel contract),
# `string` (the documented partial module-level helper hasattr
# surface — `ascii_lowercase` / `ascii_uppercase` /
# `ascii_letters` / `digits` / `hexdigits` / `octdigits` /
# `punctuation` / `whitespace` / `Formatter` / `Template` +
# the documented string.ascii_lowercase / digits / hexdigits
# value contract), `textwrap` (the documented partial module-
# level helper hasattr surface — `wrap` / `fill` / `shorten`
# / `dedent` / `indent` + the documented textwrap.dedent /
# textwrap.fill / textwrap.indent value contract), and
# `keyword` (the documented full module-level helper hasattr
# surface — `iskeyword` / `kwlist` / `softkwlist` /
# `issoftkeyword` + the documented keyword.iskeyword / kwlist
# value contract).
#
# The matching subset between mamba and CPython is the full
# `re` module hasattr surface + the match / search / findall
# / sub / split / escape value layer + the IGNORECASE integer
# value layer (the `type(re.match(...)).__name__ == "Match"`
# / `type(re.compile(...)).__name__ == "Pattern"` class-
# identity layer DIVERGES — mamba returns "re.Match" /
# "re.Pattern" — module-qualified name leak + the
# `str(re.IGNORECASE) == "re.IGNORECASE"` enum-repr layer
# DIVERGES — mamba returns "2"), the partial `string` module
# hasattr surface (`ascii_lowercase` / `ascii_uppercase` /
# `ascii_letters` / `digits` / `hexdigits` / `octdigits` /
# `punctuation` / `whitespace` / `Formatter` / `Template` —
# the `printable` sentinel DIVERGES) + the value layer, the
# partial `textwrap` module hasattr surface (`wrap` / `fill`
# / `shorten` / `dedent` / `indent` — the `TextWrapper`
# class identifier DIVERGES) + the value layer, and the
# full `keyword` module hasattr surface + the value layer.
#
# Surface in this fixture:
#   • re — full module hasattr surface (compile / match /
#     search / findall / finditer / sub / split / Pattern /
#     Match / IGNORECASE / MULTILINE / DOTALL / VERBOSE /
#     UNICODE / ASCII / escape / fullmatch / error);
#   • re.match — group(0) / group(1) / group(2) capture
#     value contract;
#   • re.search — group capture value contract;
#   • re.findall / sub / split / escape — value contract;
#   • re.IGNORECASE == 2 — integer-equivalence sentinel
#     contract;
#   • string — partial module hasattr surface
#     (ascii_lowercase / ascii_uppercase / ascii_letters /
#     digits / hexdigits / octdigits / punctuation /
#     whitespace / Formatter / Template);
#   • string.ascii_lowercase / digits / hexdigits — value
#     contract;
#   • textwrap — partial module hasattr surface (wrap /
#     fill / shorten / dedent / indent);
#   • textwrap.dedent / fill / indent — value contract;
#   • keyword — full module hasattr surface (iskeyword /
#     kwlist / softkwlist / issoftkeyword);
#   • keyword.iskeyword / kwlist — value contract.
#
# Behavioral edges that DIVERGE on mamba
# (type(re.match(...)).__name__ returns "re.Match" not
# "Match", type(re.compile(...)).__name__ returns
# "re.Pattern" not "Pattern", str(re.IGNORECASE) returns
# "2" not "re.IGNORECASE" — the RegexFlag enum repr layer
# is missing, hasattr(string, "printable") returns False,
# hasattr(textwrap, "TextWrapper") returns False) are
# covered in the matching spec fixture
# `lang_re_string_textwrap_silent`.
import re
import string
import textwrap
import keyword
from typing import Any


_ledger: list[int] = []

# 1) re — full module hasattr surface
assert hasattr(re, "compile") == True; _ledger.append(1)
assert hasattr(re, "match") == True; _ledger.append(1)
assert hasattr(re, "search") == True; _ledger.append(1)
assert hasattr(re, "findall") == True; _ledger.append(1)
assert hasattr(re, "finditer") == True; _ledger.append(1)
assert hasattr(re, "sub") == True; _ledger.append(1)
assert hasattr(re, "split") == True; _ledger.append(1)
assert hasattr(re, "Pattern") == True; _ledger.append(1)
assert hasattr(re, "Match") == True; _ledger.append(1)
assert hasattr(re, "IGNORECASE") == True; _ledger.append(1)
assert hasattr(re, "MULTILINE") == True; _ledger.append(1)
assert hasattr(re, "DOTALL") == True; _ledger.append(1)
assert hasattr(re, "VERBOSE") == True; _ledger.append(1)
assert hasattr(re, "UNICODE") == True; _ledger.append(1)
assert hasattr(re, "ASCII") == True; _ledger.append(1)
assert hasattr(re, "escape") == True; _ledger.append(1)
assert hasattr(re, "fullmatch") == True; _ledger.append(1)
assert hasattr(re, "error") == True; _ledger.append(1)

# 2) re.match — group capture value contract
_m: Any = re.match(r"(\w+)\s(\w+)", "Hello World")
assert _m is not None; _ledger.append(1)
assert _m.group() == "Hello World"; _ledger.append(1)
assert _m.group(1) == "Hello"; _ledger.append(1)
assert _m.group(2) == "World"; _ledger.append(1)

# 3) re.search — group capture value contract
_s: Any = re.search(r"\d+", "abc 123 def")
assert _s is not None; _ledger.append(1)
assert _s.group() == "123"; _ledger.append(1)

# 4) re.findall / sub / split / escape — value contract
assert re.findall(r"\d+", "a 1 b 22 c 333") == ["1", "22", "333"]; _ledger.append(1)
assert re.sub(r"\d+", "N", "a 1 b 22 c 333") == "a N b N c N"; _ledger.append(1)
assert re.split(r"\s+", "a  b   c") == ["a", "b", "c"]; _ledger.append(1)
assert re.escape("a.b") == r"a\.b"; _ledger.append(1)

# 5) re.IGNORECASE — integer-equivalence sentinel contract
assert re.IGNORECASE == 2; _ledger.append(1)

# 6) string — partial module hasattr surface
assert hasattr(string, "ascii_lowercase") == True; _ledger.append(1)
assert hasattr(string, "ascii_uppercase") == True; _ledger.append(1)
assert hasattr(string, "ascii_letters") == True; _ledger.append(1)
assert hasattr(string, "digits") == True; _ledger.append(1)
assert hasattr(string, "hexdigits") == True; _ledger.append(1)
assert hasattr(string, "octdigits") == True; _ledger.append(1)
assert hasattr(string, "punctuation") == True; _ledger.append(1)
assert hasattr(string, "whitespace") == True; _ledger.append(1)
assert hasattr(string, "Formatter") == True; _ledger.append(1)
assert hasattr(string, "Template") == True; _ledger.append(1)

# 7) string — value contract
assert string.ascii_lowercase == "abcdefghijklmnopqrstuvwxyz"; _ledger.append(1)
assert string.digits == "0123456789"; _ledger.append(1)
assert string.hexdigits == "0123456789abcdefABCDEF"; _ledger.append(1)

# 8) textwrap — partial module hasattr surface
#    (TextWrapper class identifier DIVERGES — moved to spec
#    fixture)
assert hasattr(textwrap, "wrap") == True; _ledger.append(1)
assert hasattr(textwrap, "fill") == True; _ledger.append(1)
assert hasattr(textwrap, "shorten") == True; _ledger.append(1)
assert hasattr(textwrap, "dedent") == True; _ledger.append(1)
assert hasattr(textwrap, "indent") == True; _ledger.append(1)

# 9) textwrap — value contract
assert textwrap.dedent("  a\n  b") == "a\nb"; _ledger.append(1)
assert textwrap.fill("Hello World This Is A Test", 10) == "Hello\nWorld This\nIs A Test"; _ledger.append(1)
assert textwrap.indent("a\nb", "> ") == "> a\n> b"; _ledger.append(1)

# 10) keyword — full module hasattr surface
assert hasattr(keyword, "iskeyword") == True; _ledger.append(1)
assert hasattr(keyword, "kwlist") == True; _ledger.append(1)
assert hasattr(keyword, "softkwlist") == True; _ledger.append(1)
assert hasattr(keyword, "issoftkeyword") == True; _ledger.append(1)

# 11) keyword — value contract
assert keyword.iskeyword("if") == True; _ledger.append(1)
assert keyword.iskeyword("foo") == False; _ledger.append(1)
assert "if" in keyword.kwlist; _ledger.append(1)
assert "for" in keyword.kwlist; _ledger.append(1)

# NB: type(re.match(...)).__name__ returns "re.Match" on
# mamba (not "Match" — module-qualified name leak),
# type(re.compile(...)).__name__ returns "re.Pattern" on
# mamba (not "Pattern" — module-qualified name leak),
# str(re.IGNORECASE) returns "2" on mamba (not
# "re.IGNORECASE" — the RegexFlag enum repr layer is
# missing), hasattr(string, "printable") returns False on
# mamba, hasattr(textwrap, "TextWrapper") returns False on
# mamba — all DIVERGE on mamba — moved to the divergence-
# spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_re_string_textwrap_keyword_value_ops {sum(_ledger)} asserts")
