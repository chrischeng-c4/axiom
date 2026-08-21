# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "urljoin_resolves_relative"
# subject = "urllib.parse.urljoin"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.urljoin: urljoin resolves relative references against a base per RFC 3986 sec 5.3: bare/absolute paths, scheme override, query-only, fragment-only, and ../ ./ dot-segments"""
from urllib.parse import urljoin

assert urljoin("http://a.com/b/c", "d") == "http://a.com/b/d", "rel"
assert urljoin("http://a.com/b/c", "/d") == "http://a.com/d", "abs path"
assert urljoin("http://a.com/b/c", "http://other.com/x") == \
    "http://other.com/x", "scheme override"
assert urljoin("http://a.com/b/c", "?q=1") == "http://a.com/b/c?q=1", "query only"
assert urljoin("http://a.com/b/c", "#frag") == "http://a.com/b/c#frag", "fragment only"
assert urljoin("http://a.com/b/c/", "../d") == "http://a.com/b/d", "dotdot"
assert urljoin("http://a.com/b/c/", "./d") == "http://a.com/b/c/d", "dot"

print("urljoin_resolves_relative OK")
