# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "real_world"
# case = "parse_api_walkthrough"
# subject = "urllib.parse"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse: a downstream consumer drives the full parse/unparse/join/defrag/quote/encode surface over realistic URLs and asserts each result, mirroring the legacy basic.py golden walkthrough"""
from urllib.parse import urlparse, urlsplit, urlunparse, urlunsplit, urljoin, urldefrag, parse_qs, parse_qsl, urlencode, quote, unquote, quote_plus, unquote_plus

r = urlparse("https://example.com:8080/a/b?x=1&y=2#frag")
assert r.scheme == "https" and r.netloc == "example.com:8080", f"urlparse = {r!r}"
assert r.path == "/a/b" and r.query == "x=1&y=2" and r.fragment == "frag", f"urlparse = {r!r}"

s = urlsplit("https://example.com/x?a=1")
assert s.scheme == "https" and s.path == "/x" and s.query == "a=1", f"urlsplit = {s!r}"

assert urlunparse(("https", "example.com", "/p", "", "q=1", "f")) == "https://example.com/p?q=1#f"
assert urlunsplit(("http", "h.com", "/", "", "")) == "http://h.com/"

assert urljoin("https://example.com/a/b", "../c") == "https://example.com/c"
assert urljoin("https://example.com/a/b", "https://other.com/x") == "https://other.com/x"

d = urldefrag("https://example.com/p#frag")
assert d.url == "https://example.com/p" and d.fragment == "frag", f"urldefrag = {d!r}"

assert parse_qs("a=1&b=2&a=3") == {"a": ["1", "3"], "b": ["2"]}
assert parse_qsl("a=1&b=2") == [("a", "1"), ("b", "2")]

assert urlencode({"a": 1, "b": "two"}) == "a=1&b=two"
assert urlencode([("k", "v"), ("k", "w")]) == "k=v&k=w"

assert quote("hello world/foo?bar") == "hello%20world/foo%3Fbar"
assert quote_plus("a b+c") == "a+b%2Bc"
assert unquote("hello%20world%2Ffoo") == "hello world/foo"
assert unquote_plus("a+b%20c") == "a b c"

print("parse_api_walkthrough OK")
