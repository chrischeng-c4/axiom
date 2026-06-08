# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "parse_ns_headers_netscape_cookies"
# subject = "http.cookiejar.parse_ns_headers"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.parse_ns_headers: parse_ns_headers parses Netscape Set-Cookie headers into (name, value) attribute lists, appends synthetic ('version', '0') unless an explicit version is present, and converts valid expires to a Unix timestamp"""
from http.cookiejar import parse_ns_headers

# The cookie name/value comes first; valueless attributes parse to None; a
# trailing ('version', '0') marks Netscape (non-RFC2965) cookies.
assert parse_ns_headers(["foo=bar; path=/; domain"]) == [
    [("foo", "bar"), ("path", "/"), ("domain", None), ("version", "0")]
]

# An unparseable expires value is dropped to None.
assert parse_ns_headers(["foo=bar; expires=Foo Bar 12 33:22:11 2000"]) == [
    [("foo", "bar"), ("expires", None), ("version", "0")]
]

# A bare cookie (no '=') yields a None value.
assert parse_ns_headers(["foo"]) == [[("foo", None), ("version", "0")]]

# An attribute keyword with no value.
assert parse_ns_headers(["foo=bar; expires"]) == [
    [("foo", "bar"), ("expires", None), ("version", "0")]
]

# An explicit (valueless) version suppresses the synthetic '0'.
assert parse_ns_headers(["foo=bar; version"]) == [[("foo", "bar"), ("version", None)]]

# Empty header -> no cookies at all.
assert parse_ns_headers([""]) == []

# A valid expires date is converted to a Unix timestamp; quoting is tolerated.
expires_expected = [[("foo", "bar"), ("expires", 2209069412), ("version", "0")]]
for hdr in [
    "foo=bar; expires=01 Jan 2040 22:23:32 GMT",
    'foo=bar; expires="01 Jan 2040 22:23:32 GMT"',
]:
    assert parse_ns_headers([hdr]) == expires_expected, hdr

# An explicit quoted version is case-insensitive and kept as a string.
version_expected = [[("foo", "bar"), ("version", "1")]]
for hdr in ['foo=bar; version="1"', 'foo=bar; Version="1"']:
    assert parse_ns_headers([hdr]) == version_expected, hdr

# A leading attribute named like a reserved word ("expires") is treated as the
# cookie's name/value when it is the first pair.
assert parse_ns_headers(["expires=01 Jan 2040 22:23:32 GMT"]) == [
    [("expires", "01 Jan 2040 22:23:32 GMT"), ("version", "0")]
]

print("parse_ns_headers_netscape_cookies OK")
