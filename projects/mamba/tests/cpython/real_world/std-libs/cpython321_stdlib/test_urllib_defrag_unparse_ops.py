# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_urllib_defrag_unparse_ops"
# subject = "cpython321.test_urllib_defrag_unparse_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_urllib_defrag_unparse_ops.py"
# status = "filled"
# ///
"""cpython321.test_urllib_defrag_unparse_ops: execute CPython 3.12 seed test_urllib_defrag_unparse_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `urllib.parse` surface not
# covered by `test_urllib_parse_ops`, `test_urllib_parse_advanced_ops`,
# or `test_urllib_quote_urlencode_ops`. This seed asserts
# `urldefrag` returns a (url, fragment)-named result; the
# `urlunparse` / `urlunsplit` inverse functions reconstruct the
# original URL string; and `unquote_to_bytes` decodes percent-escapes
# into a `bytes` object (not str).
from urllib.parse import (
    urldefrag,
    urlparse,
    urlsplit,
    urlunparse,
    urlunsplit,
    unquote_to_bytes,
)
_ledger: list[int] = []

# urldefrag — splits URL into url-without-fragment and fragment
r = urldefrag("https://example.com/path#frag")
assert r.url == "https://example.com/path"; _ledger.append(1)
assert r.fragment == "frag"; _ledger.append(1)

# urldefrag with no fragment leaves url intact and fragment empty
r2 = urldefrag("https://example.com/path")
assert r2.url == "https://example.com/path"; _ledger.append(1)
assert r2.fragment == ""; _ledger.append(1)

# urldefrag with empty fragment
r3 = urldefrag("https://example.com/#")
assert r3.fragment == ""; _ledger.append(1)

# urlunparse round-trip through urlparse
url = "https://example.com/path?q=1#frag"
p = urlparse(url)
assert urlunparse(p) == url; _ledger.append(1)

url2 = "http://a.com/x/y"
assert urlunparse(urlparse(url2)) == url2; _ledger.append(1)

# urlunsplit round-trip through urlsplit
s = urlsplit(url)
assert urlunsplit(s) == url; _ledger.append(1)

url3 = "https://example.com/"
assert urlunsplit(urlsplit(url3)) == url3; _ledger.append(1)

# unquote_to_bytes — produces bytes, not str
b = unquote_to_bytes("hello%20world")
assert isinstance(b, bytes); _ledger.append(1)
assert b == b"hello world"; _ledger.append(1)

# unquote_to_bytes on plain ASCII is identity (as bytes)
assert unquote_to_bytes("plain") == b"plain"; _ledger.append(1)

# unquote_to_bytes preserves non-percent characters
assert unquote_to_bytes("a%2Bb") == b"a+b"; _ledger.append(1)

# unquote_to_bytes on empty input returns empty bytes
assert unquote_to_bytes("") == b""; _ledger.append(1)

# urldefrag and urlunparse together: defrag then unparse reproduces the path part
defragged = urldefrag("https://example.com/x?y=1#z")
assert defragged.url == "https://example.com/x?y=1"; _ledger.append(1)

# urlunparse with all-empty parts produces empty url
empty_url = urlunparse(urlparse(""))
assert isinstance(empty_url, str); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_urllib_defrag_unparse_ops {sum(_ledger)} asserts")
