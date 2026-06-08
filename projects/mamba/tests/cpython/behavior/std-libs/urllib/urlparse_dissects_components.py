# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "urlparse_dissects_components"
# subject = "urllib.parse.urlparse"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.urlparse: urlparse splits a full URL into scheme/netloc/path/query/fragment and a scheme-less relative URL into just path/query"""
from urllib.parse import urlparse

p = urlparse("https://example.com:8080/path/x?a=1&b=2#frag")
assert p.scheme == "https", f"scheme = {p.scheme!r}"
assert p.netloc == "example.com:8080", f"netloc = {p.netloc!r}"
assert p.path == "/path/x", f"path = {p.path!r}"
assert p.query == "a=1&b=2", f"query = {p.query!r}"
assert p.fragment == "frag", f"fragment = {p.fragment!r}"
p2 = urlparse("/relative/path?q=1")
assert p2.scheme == "", f"rel scheme = {p2.scheme!r}"
assert p2.path == "/relative/path", f"rel path = {p2.path!r}"
assert p2.query == "q=1", f"rel query = {p2.query!r}"

print("urlparse_dissects_components OK")
