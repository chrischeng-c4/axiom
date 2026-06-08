# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_html_escape_ops"
# subject = "cpython321.test_html_escape_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_html_escape_ops.py"
# status = "filled"
# ///
"""cpython321.test_html_escape_ops: execute CPython 3.12 seed test_html_escape_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `html.escape` + `html.unescape`.
# Surface: escape replaces the five XML-significant chars
# (& < > " ') with named or numeric entities; unescape is the inverse
# for the canonical named entities (&lt; &gt; &amp; &quot; &#39;).
# Text without any escape-trigger char round-trips unchanged.
# Companion to stub/test_html.py — vendored unittest seed.
import html
_ledger: list[int] = []
# Text without escape-trigger chars is identity under escape
assert html.escape("normal text") == "normal text"; _ledger.append(1)
assert html.escape("") == ""; _ledger.append(1)
# Ampersand and angle brackets get the canonical named entity
assert html.escape("a & b") == "a &amp; b"; _ledger.append(1)
assert html.escape("<") == "&lt;"; _ledger.append(1)
assert html.escape(">") == "&gt;"; _ledger.append(1)
# unescape inverts canonical named entities
assert html.unescape("&lt;a&gt;&amp;") == "<a>&"; _ledger.append(1)
assert html.unescape("&amp;") == "&"; _ledger.append(1)
# Round-trip identity for the ampersand-containing payload
assert html.unescape(html.escape("a & b")) == "a & b"; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_html_escape_ops {sum(_ledger)} asserts")
