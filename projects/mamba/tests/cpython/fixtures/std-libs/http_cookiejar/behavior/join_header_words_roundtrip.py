# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "join_header_words_roundtrip"
# subject = "http.cookiejar.join_header_words"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.join_header_words: join_header_words is the inverse of split_header_words: it omits '=' on bare names, quotes values when needed, and normalizes spacing on round-trip"""
from http.cookiejar import split_header_words, join_header_words

# join_header_words: a bare name has no '='; values are quoted when needed.
assert join_header_words([[("foo", None), ("bar", "baz")]]) == "foo; bar=baz"
assert join_header_words([[]]) == ""

# Round-trip: split then join normalizes spacing and quoting.
ROUNDTRIP = [
    ("foo", "foo"),
    ("foo=bar", "foo=bar"),
    ("   foo   ", "foo"),
    ("foo=", 'foo=""'),
    ("foo=bar bar=baz", "foo=bar; bar=baz"),
    ("foo=bar;bar=baz", "foo=bar; bar=baz"),
    ("foo bar baz", "foo; bar; baz"),
    ("foo,,,bar", "foo, bar"),
    ("foo=bar,bar=baz", "foo=bar, bar=baz"),
    ("text/html; charset=iso-8859-1", 'text/html; charset="iso-8859-1"'),
    (
        'foo="bar"; port="80,81"; discard, bar=baz',
        'foo=bar; port="80,81"; discard, bar=baz',
    ),
]
for arg, expect in ROUNDTRIP:
    assert join_header_words(split_header_words([arg])) == expect, arg

print("join_header_words_roundtrip OK")
