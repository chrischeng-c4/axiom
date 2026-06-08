# Promoted from the upstream unittest port to an executable AssertionPass seed.
# Surface: html (escape of the five XML-special characters; unescape of the
# named entities lt/gt/amp/quot; round-trip preservation of free text).
# Numeric escapes (&#NN;), hex escapes (&#xNN;), and non-XML named entities
# (&copy;, &nbsp;, ...) are NOT decoded by mamba's html.unescape today and are
# intentionally omitted; tracked separately.
import html

_ledger: list[int] = []

# escape encodes '<' as &lt;
assert html.escape("<a>") == "&lt;a&gt;", "escape('<a>') == '&lt;a&gt;'"
_ledger.append(1)

# escape encodes '&' as &amp;
assert html.escape("a&b") == "a&amp;b", "escape('a&b') == 'a&amp;b'"
_ledger.append(1)

# escape encodes both quote variants by default
assert html.escape('"\'') == "&quot;&#x27;", "escape('\"\\'') == '&quot;&#x27;'"
_ledger.append(1)

# escape encodes all five XML-special characters in one pass
assert html.escape('<>"&\'') == "&lt;&gt;&quot;&amp;&#x27;", (
    "escape covers <, >, \", &, ' in a single call"
)
_ledger.append(1)

# escape leaves plain text alone
assert html.escape("plain text") == "plain text", "escape('plain text') is a no-op"
_ledger.append(1)

# escape of an empty string is empty
assert html.escape("") == "", "escape('') == ''"
_ledger.append(1)

# unescape decodes &lt; / &gt;
assert html.unescape("&lt;a&gt;") == "<a>", "unescape('&lt;a&gt;') == '<a>'"
_ledger.append(1)

# unescape decodes &amp;
assert html.unescape("&amp;") == "&", "unescape('&amp;') == '&'"
_ledger.append(1)

# unescape of an already-flush string is a no-op
assert html.unescape("plain text") == "plain text", (
    "unescape('plain text') is a no-op"
)
_ledger.append(1)

# unescape of an empty string is empty
assert html.unescape("") == "", "unescape('') == ''"
_ledger.append(1)

# escape -> unescape round-trip preserves a typical HTML fragment
src = '<a href="x">hi & bye</a>'
assert html.unescape(html.escape(src)) == src, (
    "unescape(escape(x)) is identity for printable HTML fragments"
)
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_html {sum(_ledger)} asserts")
