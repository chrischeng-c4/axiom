# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "urlunparse_roundtrip"
# subject = "urllib.parse.urlunparse"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlunparse: urlunparse reconstructs a 6-tuple into a URL and round-trips urlparse exactly: urlunparse(('https','example.com','/path','','q=1','')) and re-assembly of 'https://example.com/path?a=1&b=2#section'"""
from urllib.parse import urlparse, urlunparse

url = urlunparse(("https", "example.com", "/path", "", "q=1", ""))
assert url == "https://example.com/path?q=1", f"urlunparse = {url!r}"

original = "https://example.com/path?a=1&b=2#section"
reconstructed = urlunparse(urlparse(original))
assert reconstructed == original, f"round-trip = {reconstructed!r}"

print("urlunparse_roundtrip OK")
