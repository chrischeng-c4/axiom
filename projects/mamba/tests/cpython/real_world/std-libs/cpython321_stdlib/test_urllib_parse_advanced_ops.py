# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_urllib_parse_advanced_ops"
# subject = "cpython321.test_urllib_parse_advanced_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_urllib_parse_advanced_ops.py"
# status = "filled"
# ///
"""cpython321.test_urllib_parse_advanced_ops: execute CPython 3.12 seed test_urllib_parse_advanced_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for urllib.parse surfaces beyond
# test_urllib_parse_ops (which covers urlparse scheme/netloc/path/
# query/fragment, urlencode dict→query, quote/unquote round-trip).
# Surface: urlencode escapes spaces as '+' by default; urlsplit
# decomposes a URL into the same fields as urlparse but without the
# split-out params slot; parse_qs returns dict-of-lists; parse_qsl
# returns list-of-tuples; quote with explicit safe="" forces the
# slash to be percent-encoded; quote leaves the slash alone by
# default; urljoin resolves relative, absolute-path, and fully-
# qualified URLs against a base.
from urllib.parse import urlparse, urlsplit, urlencode, parse_qs, parse_qsl, quote, unquote, urljoin
_ledger: list[int] = []

# urlparse pulls scheme / netloc / path / query / fragment apart
u = urlparse("https://example.com:8080/path?q=1#frag")
assert u.scheme == "https"; _ledger.append(1)
assert u.netloc == "example.com:8080"; _ledger.append(1)
assert u.path == "/path"; _ledger.append(1)
assert u.query == "q=1"; _ledger.append(1)
assert u.fragment == "frag"; _ledger.append(1)

# urlencode escapes a literal space as '+', not %20
assert urlencode({"q": "hello world"}) == "q=hello+world"; _ledger.append(1)
# urlencode on simple ASCII key/value pairs
assert urlencode({"a": "1", "b": "two"}) == "a=1&b=two"; _ledger.append(1)
# Single-key encoding
assert urlencode({"k": "v"}) == "k=v"; _ledger.append(1)

# parse_qs returns a dict-of-lists (one list per key)
assert parse_qs("a=1&b=2") == {"a": ["1"], "b": ["2"]}; _ledger.append(1)
# Repeated key accumulates into the list
assert parse_qs("a=1&a=2") == {"a": ["1", "2"]}; _ledger.append(1)

# parse_qsl returns a flat list of (key, value) tuples preserving order
assert parse_qsl("a=1&b=2") == [("a", "1"), ("b", "2")]; _ledger.append(1)
assert parse_qsl("a=1&a=2") == [("a", "1"), ("a", "2")]; _ledger.append(1)

# quote leaves '/' alone by default — it's in the safe set
assert quote("a/b") == "a/b"; _ledger.append(1)
# quote with safe="" forces '/' to be percent-encoded
assert quote("a/b", safe="") == "a%2Fb"; _ledger.append(1)
# quote escapes spaces as %20 (the path-style default, unlike urlencode)
assert quote("hello world") == "hello%20world"; _ledger.append(1)
# unquote reverses %20 → space
assert unquote("hello%20world") == "hello world"; _ledger.append(1)
# unquote of a multi-byte UTF-8 sequence yields the original char
assert unquote("%E2%86%92") == "→"; _ledger.append(1)

# urljoin resolves a relative path against a base ending in '/'
assert urljoin("http://a.com/x/", "y") == "http://a.com/x/y"; _ledger.append(1)
# urljoin with an absolute-path reference replaces the path
assert urljoin("http://a.com/x/y", "/z") == "http://a.com/z"; _ledger.append(1)
# urljoin with a fully-qualified URL replaces everything
assert urljoin("http://a.com/", "http://b.com/x") == "http://b.com/x"; _ledger.append(1)

# urlsplit decomposes a URL the same as urlparse but with a leaner
# 5-field tuple (no separate `params` slot)
s = urlsplit("https://example.com/path?q=1")
assert s.scheme == "https"; _ledger.append(1)
assert s.netloc == "example.com"; _ledger.append(1)
assert s.path == "/path"; _ledger.append(1)
assert s.query == "q=1"; _ledger.append(1)

# Round-trip: parse_qsl into a dict and back via urlencode
qs = "a=1&b=two"
pairs = parse_qsl(qs)
encoded = urlencode(dict(pairs))
assert encoded == "a=1&b=two"; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_urllib_parse_advanced_ops {sum(_ledger)} asserts")
