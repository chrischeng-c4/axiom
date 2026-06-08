# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "urljoin_relative_navigation"
# subject = "urllib.parse.urljoin"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urljoin: urljoin resolves relative references against a base: 'c' replaces the last segment, '/c' is an absolute path, a full URL wins outright, and '../' segments walk up the path"""
from urllib.parse import urljoin

assert urljoin("http://example.com/a/b", "c") == "http://example.com/a/c", "relative replaces last segment"
assert urljoin("http://example.com/a/b", "/c") == "http://example.com/c", "absolute path"
assert urljoin("http://a.com/", "http://b.com/") == "http://b.com/", "full URL wins"
assert urljoin("http://a.com/b/c/d", "../e") == "http://a.com/b/e", "one updir"
assert urljoin("http://a.com/b/c/", "../../d") == "http://a.com/d", "two updirs"

print("urljoin_relative_navigation OK")
