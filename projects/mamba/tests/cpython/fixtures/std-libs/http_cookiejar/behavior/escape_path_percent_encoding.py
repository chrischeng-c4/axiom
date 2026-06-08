# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "escape_path_percent_encoding"
# subject = "http.cookiejar.escape_path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.escape_path: escape_path %-escapes unsafe bytes (upper-case hex), keeps already-safe characters, and UTF-8-encodes non-ASCII"""
from http.cookiejar import escape_path

assert escape_path("/foo%2f/bar") == "/foo%2F/bar"
assert escape_path("/foo/bar&") == "/foo/bar&"
assert escape_path("/foo\x19/bar") == "/foo%19/bar"
assert escape_path("/}foo/bar") == "/%7Dfoo/bar"
assert escape_path("/foo/barü") == "/foo/bar%C3%BC"

print("escape_path_percent_encoding OK")
