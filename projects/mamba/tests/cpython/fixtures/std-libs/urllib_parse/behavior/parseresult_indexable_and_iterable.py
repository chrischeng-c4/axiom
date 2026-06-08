# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "parseresult_indexable_and_iterable"
# subject = "urllib.parse.urlparse"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlparse: ParseResult is a 6-element named tuple: urlparse('https://example.com/path')[0]=='https', [2]=='/path', and list() of it has length 6"""
from urllib.parse import urlparse

r = urlparse("https://example.com/path")
assert r[0] == "https", f"scheme by index = {r[0]!r}"
assert r[2] == "/path", f"path by index = {r[2]!r}"
parts = list(r)
assert len(parts) == 6, f"six parts = {len(parts)!r}"

print("parseresult_indexable_and_iterable OK")
