# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_html_entities_ops"
# subject = "cpython321.test_html_entities_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_html_entities_ops.py"
# status = "filled"
# ///
"""cpython321.test_html_entities_ops: execute CPython 3.12 seed test_html_entities_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the html-module escape/unescape
# surface. Surface: html.escape replaces &/</> with the canonical
# &amp;/&lt;/&gt; entities; the default form (quote=True) also escapes
# single and double quotes to &#x27;/&quot;; plain text without metas
# is passed through unchanged; html.unescape inverts &amp;/&lt;/&gt;/
# &quot;/&#x27; on representative samples; round-trips through
# html.unescape(html.escape(s)) restore the original for a string
# containing every metacharacter. Companion to test_html_escape_ops
# (which covers the escape side in more depth).
import html
_ledger: list[int] = []

# escape — angle brackets and ampersand
assert html.escape("<b>hi</b>") == "&lt;b&gt;hi&lt;/b&gt;"; _ledger.append(1)
assert html.escape("a & b") == "a &amp; b"; _ledger.append(1)
assert html.escape("<") == "&lt;"; _ledger.append(1)
assert html.escape(">") == "&gt;"; _ledger.append(1)
assert html.escape("&") == "&amp;"; _ledger.append(1)

# escape — default form escapes both quote kinds
assert html.escape("'hello'") == "&#x27;hello&#x27;"; _ledger.append(1)
assert html.escape('"q"') == "&quot;q&quot;"; _ledger.append(1)

# escape — plain text passes through
assert html.escape("plain text") == "plain text"; _ledger.append(1)

# escape with quote=False — & still escaped, quotes left alone
assert html.escape("a&b", quote=False) == "a&amp;b"; _ledger.append(1)

# unescape — basic entities
assert html.unescape("&lt;b&gt;") == "<b>"; _ledger.append(1)
assert html.unescape("a &amp; b") == "a & b"; _ledger.append(1)
assert html.unescape("&quot;hi&quot;") == '"hi"'; _ledger.append(1)
assert html.unescape("&#x27;hi&#x27;") == "'hi'"; _ledger.append(1)
assert html.unescape("plain") == "plain"; _ledger.append(1)
assert html.unescape("&amp;") == "&"; _ledger.append(1)
assert html.unescape("&lt;") == "<"; _ledger.append(1)
assert html.unescape("&gt;") == ">"; _ledger.append(1)

# round-trip — every metacharacter restored
s = "<div>'hello' & \"world\"</div>"
assert html.unescape(html.escape(s)) == s; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_html_entities_ops {sum(_ledger)} asserts")
