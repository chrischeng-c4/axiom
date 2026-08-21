# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "real_world"
# case = "parse_api_walkthrough"
# subject = "urllib.parse"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse: a downstream consumer drives quote/quote_plus/unquote/unquote_plus/urlencode/urlparse/urlunparse/urljoin/parse_qs/parse_qsl together over realistic URLs and query strings, asserting each result"""
from urllib.parse import quote, quote_plus, unquote, unquote_plus, urlencode, urlparse, urlunparse, urljoin, parse_qs, parse_qsl

assert quote("a b/c") == "a%20b/c"
assert quote_plus("a b") == "a+b"
assert unquote("a%20b") == "a b"
assert unquote_plus("a+b") == "a b"
assert urlencode({"x": "1"}) == "x=1"

p = urlparse("https://host:80/p?k=v#f")
assert (p.scheme, p.netloc, p.path, p.query, p.fragment) == \
    ("https", "host:80", "/p", "k=v", "f"), repr(p)
assert urlunparse(("https", "host", "/p", "", "k=v", "f")) == \
    "https://host/p?k=v#f"
assert urljoin("http://a/b/c", "d") == "http://a/b/d"
assert parse_qs("a=1&a=2&b=3") == {"a": ["1", "2"], "b": ["3"]}
assert parse_qsl("a=1&b=2") == [("a", "1"), ("b", "2")]

print("parse_api_walkthrough OK")
