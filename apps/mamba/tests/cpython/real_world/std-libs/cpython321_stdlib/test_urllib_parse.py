# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_urllib_parse"
# subject = "cpython321.test_urllib_parse"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_urllib_parse.py"
# status = "filled"
# ///
"""cpython321.test_urllib_parse: execute CPython 3.12 seed test_urllib_parse"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Promoted from the upstream unittest port to an executable AssertionPass seed.
# Surface: urllib.parse — urlparse (attribute access), urlunparse, quote /
# unquote / quote_plus / unquote_plus, urlencode (dict + list-of-pairs),
# parse_qs / parse_qsl, urljoin.
# ParseResult tuple-indexing (r[0]) returns None on mamba today and is
# intentionally NOT exercised here; attribute access works and is the
# common idiom. ParseResult.hostname / .port (derived properties) are not
# implemented and are also omitted; tracked separately.
# `import urllib, urllib.parse` patterns hit the dotted-import quirk on
# mamba — we use `from urllib.parse import ...` explicitly.
from urllib.parse import (
    urlparse,
    urlunparse,
    quote,
    unquote,
    quote_plus,
    unquote_plus,
    urlencode,
    parse_qs,
    parse_qsl,
    urljoin,
)

_ledger: list[int] = []

# urlparse splits a fully-qualified URL into its component fields
_r = urlparse("http://www.example.com/path?q=1#frag")
assert _r.scheme == "http", "urlparse scheme"
_ledger.append(1)

assert _r.netloc == "www.example.com", "urlparse netloc"
_ledger.append(1)

assert _r.path == "/path", "urlparse path"
_ledger.append(1)

assert _r.query == "q=1", "urlparse query"
_ledger.append(1)

assert _r.fragment == "frag", "urlparse fragment"
_ledger.append(1)

# urlparse on a URL without a path leaves path empty
_r2 = urlparse("http://example.com")
assert _r2.scheme == "http" and _r2.netloc == "example.com" and _r2.path == "", (
    "urlparse('http://example.com') -> scheme=http netloc=example.com path=''"
)
_ledger.append(1)

# urlunparse rebuilds a URL from its 6-tuple
assert urlunparse(("http", "example.com", "/p", "", "q=1", "f")) == "http://example.com/p?q=1#f", (
    "urlunparse round-trip"
)
_ledger.append(1)

# quote percent-encodes a space but, by default, leaves '/' alone
assert quote("hello world") == "hello%20world", "quote ' ' -> %20"
_ledger.append(1)

assert quote("hello/world") == "hello/world", "quote leaves '/' alone by default"
_ledger.append(1)

# quote(safe='') escapes '/' as well
assert quote("hello/world", safe="") == "hello%2Fworld", "quote(safe='') escapes '/'"
_ledger.append(1)

# unquote inverts %-encoding
assert unquote("hello%20world") == "hello world", "unquote %20 -> ' '"
_ledger.append(1)

# quote_plus encodes spaces as '+' (form-encoded variant)
assert quote_plus("hello world") == "hello+world", "quote_plus ' ' -> '+'"
_ledger.append(1)

# unquote_plus inverts quote_plus
assert unquote_plus("hello+world") == "hello world", "unquote_plus '+' -> ' '"
_ledger.append(1)

# urlencode encodes a dict of query parameters
assert urlencode({"a": "1", "b": "2"}) == "a=1&b=2", "urlencode dict"
_ledger.append(1)

# urlencode encodes a list of (key, value) pairs in order
assert urlencode([("a", "1"), ("b", "2")]) == "a=1&b=2", "urlencode list-of-pairs"
_ledger.append(1)

# parse_qs decodes a query string into a dict of lists
assert parse_qs("a=1&b=2") == {"a": ["1"], "b": ["2"]}, "parse_qs"
_ledger.append(1)

# parse_qsl decodes a query string into an ordered list of pairs
assert parse_qsl("a=1&b=2") == [("a", "1"), ("b", "2")], "parse_qsl"
_ledger.append(1)

# urljoin resolves a relative reference against an absolute base
assert urljoin("http://example.com/path/", "x") == "http://example.com/path/x", (
    "urljoin('.../path/', 'x') -> '.../path/x'"
)
_ledger.append(1)

# urljoin resolves a root-relative reference against an absolute base
assert urljoin("http://example.com/path/", "/y") == "http://example.com/y", (
    "urljoin('.../path/', '/y') -> '.../y'"
)
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_urllib_parse {sum(_ledger)} asserts")
