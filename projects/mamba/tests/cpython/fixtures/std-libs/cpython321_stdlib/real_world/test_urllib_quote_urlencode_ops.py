# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_urllib_quote_urlencode_ops"
# subject = "cpython321.test_urllib_quote_urlencode_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_urllib_quote_urlencode_ops.py"
# status = "filled"
# ///
"""cpython321.test_urllib_quote_urlencode_ops: execute CPython 3.12 seed test_urllib_quote_urlencode_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the urllib.parse quote/
# urlencode surface. Surface: `urllib.parse.quote(s)` percent-
# encodes URL-reserved characters but leaves `/` (the default
# safe character) and ASCII-letters/digits alone, percent-encodes
# non-ASCII through utf-8 ("é" -> "%C3%A9"), and is the inverse
# of `urllib.parse.unquote(s)`; `quote_plus` is `quote` with `+`
# substituting space; `urlencode` over a `{k: v}` mapping or a
# `[(k, v)]` list-of-pairs builds the canonical
# `"k1=v1&k2=v2..."` query string with spaces encoded as `+` and
# `&`/`=` percent-encoded inside values; `parse_qs` returns a
# `{k: [v, ...]}` dict and `parse_qsl` returns the matching
# `[(k, v), ...]` list; `urlparse` decomposes a full URL into
# scheme/netloc/path/query/fragment, and `urlunparse` is its
# round-trip inverse; `urljoin(base, rel)` resolves a relative
# URL against a base URL.
import urllib.parse as up
_ledger: list[int] = []

# quote — default safe="/" passes "/" through untouched
assert up.quote("hello world") == "hello%20world"; _ledger.append(1)
assert up.quote("a/b/c") == "a/b/c"; _ledger.append(1)
assert up.quote("?&=") == "%3F%26%3D"; _ledger.append(1)
assert up.quote("foo+bar") == "foo%2Bbar"; _ledger.append(1)
assert up.quote("") == ""; _ledger.append(1)
assert up.quote("abc") == "abc"; _ledger.append(1)
assert up.quote("é") == "%C3%A9"; _ledger.append(1)

# unquote — inverse of quote
assert up.unquote("hello%20world") == "hello world"; _ledger.append(1)
assert up.unquote("a%2Fb%2Fc") == "a/b/c"; _ledger.append(1)
assert up.unquote("%3F%26%3D") == "?&="; _ledger.append(1)
assert up.unquote("%C3%A9") == "é"; _ledger.append(1)
assert up.unquote("plain") == "plain"; _ledger.append(1)
assert up.unquote("") == ""; _ledger.append(1)
assert up.unquote(up.quote("hello world")) == "hello world"; _ledger.append(1)

# quote_plus / unquote_plus — `+` substitutes space
assert up.quote_plus("hello world") == "hello+world"; _ledger.append(1)
assert up.quote_plus("a&b=c") == "a%26b%3Dc"; _ledger.append(1)
assert up.unquote_plus("hello+world") == "hello world"; _ledger.append(1)
assert up.unquote_plus("a%26b%3Dc") == "a&b=c"; _ledger.append(1)

# urlencode — mapping / list-of-pairs / spaces / empty
assert up.urlencode({"a": 1, "b": 2}) in ("a=1&b=2", "b=2&a=1"); _ledger.append(1)
assert up.urlencode({"name": "John Doe"}) == "name=John+Doe"; _ledger.append(1)
assert up.urlencode([("a", 1), ("b", 2)]) == "a=1&b=2"; _ledger.append(1)
assert up.urlencode({}) == ""; _ledger.append(1)

# parse_qs / parse_qsl — query-string parsing
assert up.parse_qs("a=1&b=2") == {"a": ["1"], "b": ["2"]}; _ledger.append(1)
assert up.parse_qs("") == {}; _ledger.append(1)
assert up.parse_qsl("a=1&b=2") == [("a", "1"), ("b", "2")]; _ledger.append(1)

# urlparse — split a full URL into components
u = up.urlparse("https://example.com:8080/path/file?q=1#frag")
assert u.scheme == "https"; _ledger.append(1)
assert u.netloc == "example.com:8080"; _ledger.append(1)
assert u.path == "/path/file"; _ledger.append(1)
assert u.query == "q=1"; _ledger.append(1)
assert u.fragment == "frag"; _ledger.append(1)

# urlunparse — round-trip back to the original URL
assert up.urlunparse(u) == "https://example.com:8080/path/file?q=1#frag"; _ledger.append(1)

# urljoin — resolve relative URL against base
assert up.urljoin("http://example.com/a/", "b") == "http://example.com/a/b"; _ledger.append(1)
assert up.urljoin("http://example.com/a/b", "../c") == "http://example.com/c"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_urllib_quote_urlencode_ops {sum(_ledger)} asserts")
