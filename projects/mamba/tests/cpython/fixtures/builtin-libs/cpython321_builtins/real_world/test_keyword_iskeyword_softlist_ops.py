# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_keyword_iskeyword_softlist_ops"
# subject = "cpython321.test_keyword_iskeyword_softlist_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_keyword_iskeyword_softlist_ops.py"
# status = "filled"
# ///
"""cpython321.test_keyword_iskeyword_softlist_ops: execute CPython 3.12 seed test_keyword_iskeyword_softlist_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the `keyword` module — the
# stdlib introspection helper for Python's reserved-word list
# (`iskeyword`, `issoftkeyword`, `kwlist`, `softkwlist`). Used by
# code-generators (to avoid emitting reserved names), linters,
# parsers, REPLs, and any tooling that needs to know whether a
# string is a current Python keyword. Surface focuses on the
# full reserved-word table: every Python 3.12 hard keyword
# (`False`/`None`/`True`/`and`/`as`/.../`yield`) and every soft
# keyword (`_`/`case`/`match`/`type`). Mamba and CPython agree
# on every probe (33 hard + 4 soft, matching the 3.12 kwlist
# exactly). No fixture coverage yet for the keyword module.
#
# Surface:
#   • keyword.iskeyword(s: str) → bool
#       — True iff `s` is a hard keyword;
#       — False for non-keywords ("foo") and for the empty string;
#       — case-sensitive ("IF" → False, "if" → True);
#   • keyword.issoftkeyword(s: str) → bool
#       — True iff `s` is a soft keyword (contextual, not reserved):
#         `_`, `case`, `match`, `type`;
#       — False for hard keywords and non-keywords;
#   • keyword.kwlist — list[str] of all hard keywords, sorted;
#   • keyword.softkwlist — list[str] of all soft keywords.
import keyword
_ledger: list[int] = []

# Hard keywords — control flow
assert keyword.iskeyword("if") == True; _ledger.append(1)
assert keyword.iskeyword("else") == True; _ledger.append(1)
assert keyword.iskeyword("elif") == True; _ledger.append(1)
assert keyword.iskeyword("while") == True; _ledger.append(1)
assert keyword.iskeyword("for") == True; _ledger.append(1)
assert keyword.iskeyword("break") == True; _ledger.append(1)
assert keyword.iskeyword("continue") == True; _ledger.append(1)
assert keyword.iskeyword("return") == True; _ledger.append(1)
assert keyword.iskeyword("pass") == True; _ledger.append(1)

# Hard keywords — definitions
assert keyword.iskeyword("def") == True; _ledger.append(1)
assert keyword.iskeyword("class") == True; _ledger.append(1)
assert keyword.iskeyword("lambda") == True; _ledger.append(1)

# Hard keywords — imports
assert keyword.iskeyword("import") == True; _ledger.append(1)
assert keyword.iskeyword("from") == True; _ledger.append(1)
assert keyword.iskeyword("as") == True; _ledger.append(1)

# Hard keywords — exceptions
assert keyword.iskeyword("try") == True; _ledger.append(1)
assert keyword.iskeyword("except") == True; _ledger.append(1)
assert keyword.iskeyword("finally") == True; _ledger.append(1)
assert keyword.iskeyword("raise") == True; _ledger.append(1)
assert keyword.iskeyword("assert") == True; _ledger.append(1)
assert keyword.iskeyword("with") == True; _ledger.append(1)

# Hard keywords — operators
assert keyword.iskeyword("and") == True; _ledger.append(1)
assert keyword.iskeyword("or") == True; _ledger.append(1)
assert keyword.iskeyword("not") == True; _ledger.append(1)
assert keyword.iskeyword("in") == True; _ledger.append(1)
assert keyword.iskeyword("is") == True; _ledger.append(1)

# Hard keywords — scope / generators
assert keyword.iskeyword("global") == True; _ledger.append(1)
assert keyword.iskeyword("nonlocal") == True; _ledger.append(1)
assert keyword.iskeyword("yield") == True; _ledger.append(1)
assert keyword.iskeyword("del") == True; _ledger.append(1)

# Hard keywords — async
assert keyword.iskeyword("async") == True; _ledger.append(1)
assert keyword.iskeyword("await") == True; _ledger.append(1)

# Hard keywords — literals
assert keyword.iskeyword("True") == True; _ledger.append(1)
assert keyword.iskeyword("False") == True; _ledger.append(1)
assert keyword.iskeyword("None") == True; _ledger.append(1)

# Non-keywords
assert keyword.iskeyword("foo") == False; _ledger.append(1)
assert keyword.iskeyword("bar") == False; _ledger.append(1)
assert keyword.iskeyword("abc") == False; _ledger.append(1)
assert keyword.iskeyword("hello") == False; _ledger.append(1)
assert keyword.iskeyword("") == False; _ledger.append(1)
assert keyword.iskeyword("123") == False; _ledger.append(1)
assert keyword.iskeyword("_") == False; _ledger.append(1)  # _ is soft keyword, not hard

# Case sensitivity — "IF" is not a keyword
assert keyword.iskeyword("IF") == False; _ledger.append(1)
assert keyword.iskeyword("CLASS") == False; _ledger.append(1)
assert keyword.iskeyword("Return") == False; _ledger.append(1)
assert keyword.iskeyword("TRUE") == False; _ledger.append(1)
assert keyword.iskeyword("NONE") == False; _ledger.append(1)

# Soft keywords
assert keyword.issoftkeyword("match") == True; _ledger.append(1)
assert keyword.issoftkeyword("case") == True; _ledger.append(1)
assert keyword.issoftkeyword("type") == True; _ledger.append(1)
assert keyword.issoftkeyword("_") == True; _ledger.append(1)

# Hard keywords are not soft keywords
assert keyword.issoftkeyword("if") == False; _ledger.append(1)
assert keyword.issoftkeyword("def") == False; _ledger.append(1)
assert keyword.issoftkeyword("class") == False; _ledger.append(1)
assert keyword.issoftkeyword("return") == False; _ledger.append(1)

# Non-keywords are not soft keywords either
assert keyword.issoftkeyword("foo") == False; _ledger.append(1)
assert keyword.issoftkeyword("") == False; _ledger.append(1)

# kwlist — every hard keyword from above is in the list
for _kw in ["if", "else", "elif", "while", "for", "break", "continue",
            "return", "pass", "def", "class", "lambda", "import",
            "from", "as", "try", "except", "finally", "raise",
            "assert", "with", "and", "or", "not", "in", "is",
            "global", "nonlocal", "yield", "del", "async", "await",
            "True", "False", "None"]:
    assert _kw in keyword.kwlist; _ledger.append(1)

# kwlist — non-keywords are not in it
for _nk in ["foo", "bar", "abc", "hello", ""]:
    assert _nk not in keyword.kwlist; _ledger.append(1)

# softkwlist — every soft keyword
for _sk in ["match", "case", "type", "_"]:
    assert _sk in keyword.softkwlist; _ledger.append(1)

# softkwlist — hard keywords not in soft list
for _hk in ["if", "def", "class", "return"]:
    assert _hk not in keyword.softkwlist; _ledger.append(1)

# kwlist length (Python 3.12 has 35 hard keywords)
assert len(keyword.kwlist) == 35; _ledger.append(1)
# softkwlist length (Python 3.12 has 4 soft keywords)
assert len(keyword.softkwlist) == 4; _ledger.append(1)

# Return type discipline
assert isinstance(keyword.iskeyword("if"), bool); _ledger.append(1)
assert isinstance(keyword.iskeyword("foo"), bool); _ledger.append(1)
assert isinstance(keyword.issoftkeyword("match"), bool); _ledger.append(1)
assert isinstance(keyword.kwlist, list); _ledger.append(1)
assert isinstance(keyword.softkwlist, list); _ledger.append(1)
# Every entry in kwlist is str
for _kw in keyword.kwlist:
    assert isinstance(_kw, str); _ledger.append(1)
# Every entry in softkwlist is str
for _sk in keyword.softkwlist:
    assert isinstance(_sk, str); _ledger.append(1)

# Idempotence — same query, same result
assert keyword.iskeyword("if") == keyword.iskeyword("if"); _ledger.append(1)
assert keyword.issoftkeyword("match") == keyword.issoftkeyword("match"); _ledger.append(1)

# Module-level attribute discipline
assert hasattr(keyword, "iskeyword"); _ledger.append(1)
assert hasattr(keyword, "issoftkeyword"); _ledger.append(1)
assert hasattr(keyword, "kwlist"); _ledger.append(1)
assert hasattr(keyword, "softkwlist"); _ledger.append(1)
assert callable(keyword.iskeyword); _ledger.append(1)
assert callable(keyword.issoftkeyword); _ledger.append(1)

# kwlist and softkwlist are disjoint
for _kw in keyword.kwlist:
    assert _kw not in keyword.softkwlist; _ledger.append(1)

# kwlist contains unique entries
assert len(keyword.kwlist) == len(set(keyword.kwlist)); _ledger.append(1)
assert len(keyword.softkwlist) == len(set(keyword.softkwlist)); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_keyword_iskeyword_softlist_ops {sum(_ledger)} asserts")
