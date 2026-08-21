# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "urlunparse_reconstructs"
# subject = "urllib.parse.urlunparse"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.urlunparse: urlunparse reassembles a 6-tuple (scheme, netloc, path, params, query, fragment) into the canonical URL string"""
from urllib.parse import urlunparse

parts = ("https", "example.com", "/path", "", "q=1", "frag")
url = urlunparse(parts)
assert url == "https://example.com/path?q=1#frag", f"urlunparse = {url!r}"

print("urlunparse_reconstructs OK")
