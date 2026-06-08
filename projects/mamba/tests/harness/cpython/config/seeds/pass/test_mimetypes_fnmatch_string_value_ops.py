# Operational AssertionPass seed for the value contract of three
# bootstrap stdlib modules that drive every HTTP / glob / text
# pipeline: `mimetypes` (the guess_type / guess_extension
# IANA-mapped MIME-type bidirectional surface), `fnmatch` (the
# fnmatch / filter / translate / fnmatchcase glob-pattern
# predicates), and `string` (the documented ascii_lowercase /
# ascii_uppercase / ascii_letters / digits / hexdigits /
# octdigits / punctuation character-class constants).
#
# The matching subset between mamba and CPython is the
# documented-value layer: mimetypes.guess_type returns the IANA
# MIME-type 2-tuple for known extensions and (None, None) for
# unknown; mimetypes.guess_extension returns the canonical
# extension for known MIME types; mimetypes.inited is a bool;
# fnmatch.fnmatch / fnmatchcase return the documented boolean for
# `*` / `?` / literal glob patterns; fnmatch.filter returns the
# subset of names matching a pattern; fnmatch.translate returns
# a regex `str`; string.ascii_lowercase / ascii_uppercase /
# ascii_letters / digits / hexdigits / octdigits / punctuation
# carry the documented ASCII character sequences.
#
# Surface in this fixture:
#   • mimetypes.guess_type("a.txt") == ("text/plain", None);
#   • mimetypes.guess_type("a.html") == ("text/html", None);
#   • mimetypes.guess_type("a.json") == ("application/json",
#     None);
#   • mimetypes.guess_type("a.zip") == ("application/zip", None);
#   • mimetypes.guess_type("a.unknown") == (None, None);
#   • mimetypes.guess_extension("text/plain") == ".txt";
#   • mimetypes.guess_extension("text/html") == ".html";
#   • mimetypes.guess_extension("application/json") == ".json";
#   • type(mimetypes.inited).__name__ == "bool";
#   • fnmatch.fnmatch("a.py", "*.py") is True;
#   • fnmatch.fnmatch("a.txt", "*.py") is False;
#   • fnmatch.fnmatch("abc", "*") is True;
#   • fnmatch.fnmatch("a.py", "?.py") is True;
#   • fnmatch.filter(["a.py","b.txt","c.py"], "*.py") ==
#     ["a.py", "c.py"];
#   • type(fnmatch.translate("*.py")) is str;
#   • fnmatch.fnmatchcase("Abc", "Abc") is True;
#   • string.ascii_lowercase ==
#     "abcdefghijklmnopqrstuvwxyz";
#   • string.ascii_uppercase ==
#     "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
#   • string.ascii_letters ==
#     "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
#   • string.digits == "0123456789";
#   • string.hexdigits == "0123456789abcdefABCDEF";
#   • string.octdigits == "01234567";
#   • string.punctuation ==
#     "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~".
#
# Behavioral edges that DIVERGE on mamba (mimetypes.MimeTypes
# class identity, string.printable length, string.Template
# instance substitute, string.Template / Formatter class identity,
# string.capwords, textwrap.fill / wrap / dedent / shorten,
# ast.parse mode="eval" returning Expression, ast.literal_eval on
# containers, ast.AST / Module / Expression / BinOp class
# identity, ast.dump representation, pprint.pformat single-line
# layout, pprint.PrettyPrinter class identity + isreadable /
# isrecursive helpers) are covered in
# `lang_string_textwrap_ast_pprint_silent`.
import mimetypes
import fnmatch
import string

_ledger: list[int] = []

# 1) mimetypes.guess_type — IANA MIME-type bidirectional surface
assert mimetypes.guess_type("a.txt") == ("text/plain", None); _ledger.append(1)
assert mimetypes.guess_type("a.html") == ("text/html", None); _ledger.append(1)
assert mimetypes.guess_type("a.json") == ("application/json", None); _ledger.append(1)
assert mimetypes.guess_type("a.zip") == ("application/zip", None); _ledger.append(1)
assert mimetypes.guess_type("a.unknown") == (None, None); _ledger.append(1)

# 2) mimetypes.guess_extension — MIME-type → canonical extension
assert mimetypes.guess_extension("text/plain") == ".txt"; _ledger.append(1)
assert mimetypes.guess_extension("text/html") == ".html"; _ledger.append(1)
assert mimetypes.guess_extension("application/json") == ".json"; _ledger.append(1)

# 3) mimetypes.inited — documented bool sentinel
assert type(mimetypes.inited).__name__ == "bool"; _ledger.append(1)

# 4) fnmatch.fnmatch — glob-pattern predicate
assert fnmatch.fnmatch("a.py", "*.py") == True; _ledger.append(1)
assert fnmatch.fnmatch("a.txt", "*.py") == False; _ledger.append(1)
assert fnmatch.fnmatch("abc", "*") == True; _ledger.append(1)
assert fnmatch.fnmatch("a.py", "?.py") == True; _ledger.append(1)
assert fnmatch.fnmatch("ab.py", "?.py") == False; _ledger.append(1)

# 5) fnmatch.filter — multi-name glob filter
assert fnmatch.filter(["a.py", "b.txt", "c.py"], "*.py") == ["a.py", "c.py"]; _ledger.append(1)

# 6) fnmatch.translate — glob → regex string
assert type(fnmatch.translate("*.py")).__name__ == "str"; _ledger.append(1)

# 7) fnmatch.fnmatchcase — case-sensitive exact match
assert fnmatch.fnmatchcase("Abc", "Abc") == True; _ledger.append(1)
assert fnmatch.fnmatchcase("abc", "Abc") == False; _ledger.append(1)

# 8) string — ASCII alphabetic constants
assert string.ascii_lowercase == "abcdefghijklmnopqrstuvwxyz"; _ledger.append(1)
assert string.ascii_uppercase == "ABCDEFGHIJKLMNOPQRSTUVWXYZ"; _ledger.append(1)
assert string.ascii_letters == "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"; _ledger.append(1)

# 9) string — numeric / hex / oct constants
assert string.digits == "0123456789"; _ledger.append(1)
assert string.hexdigits == "0123456789abcdefABCDEF"; _ledger.append(1)
assert string.octdigits == "01234567"; _ledger.append(1)

# 10) string.punctuation — POSIX punctuation
assert string.punctuation == "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~"; _ledger.append(1)

# 11) hasattr surface — module-level helpers
assert hasattr(mimetypes, "guess_type"); _ledger.append(1)
assert hasattr(mimetypes, "guess_extension"); _ledger.append(1)
assert hasattr(fnmatch, "fnmatch"); _ledger.append(1)
assert hasattr(fnmatch, "filter"); _ledger.append(1)
assert hasattr(string, "ascii_letters"); _ledger.append(1)
assert hasattr(string, "digits"); _ledger.append(1)

# NB: mimetypes.MimeTypes class identity, string.printable length,
# string.Template instance .substitute, string.Template /
# Formatter class identity, string.capwords, textwrap.fill / wrap
# / dedent / shorten, ast.parse mode="eval" returning Expression,
# ast.literal_eval on list / tuple / dict, ast.AST / Module /
# Expression / BinOp class identity, ast.dump representation,
# pprint.pformat single-line layout, pprint.PrettyPrinter class
# identity + isreadable / isrecursive helpers all DIVERGE on
# mamba — moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_mimetypes_fnmatch_string_value_ops {sum(_ledger)} asserts")
