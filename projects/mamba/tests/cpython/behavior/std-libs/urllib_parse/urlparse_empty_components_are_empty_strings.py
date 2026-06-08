# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "urlparse_empty_components_are_empty_strings"
# subject = "urllib.parse.urlparse"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlparse: absent query/fragment come back as '' (not None) while a missing username comes back as None: urlparse('http://example.com/') has query=='' fragment=='' username is None"""
from urllib.parse import urlparse

r = urlparse("http://example.com/")
assert r.query == "", f"empty query = {r.query!r}"
assert r.fragment == "", f"empty fragment = {r.fragment!r}"
assert r.username is None, f"no username = {r.username!r}"
assert r.port is None, f"no port = {r.port!r}"

print("urlparse_empty_components_are_empty_strings OK")
