# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "parseresult_named_fields"
# subject = "urllib.parse.urlparse"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.urlparse: the ParseResult exposes scheme/netloc/path/query/fragment plus the derived hostname and port (int) for an authority with a port"""
from urllib.parse import urlparse

p = urlparse("https://example.com:8080/path?q=1#frag")
assert p.scheme == "https", f"scheme = {p.scheme!r}"
assert p.netloc == "example.com:8080", f"netloc = {p.netloc!r}"
assert p.path == "/path", f"path = {p.path!r}"
assert p.query == "q=1", f"query = {p.query!r}"
assert p.fragment == "frag", f"fragment = {p.fragment!r}"
assert p.hostname == "example.com", f"hostname = {p.hostname!r}"
assert p.port == 8080, f"port = {p.port!r}"

print("parseresult_named_fields OK")
