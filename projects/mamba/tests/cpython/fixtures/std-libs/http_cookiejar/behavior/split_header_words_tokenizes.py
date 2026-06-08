# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "split_header_words_tokenizes"
# subject = "http.cookiejar.split_header_words"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.split_header_words: split_header_words splits each header on ',' into groups of (name, value) pairs; bare tokens get value None, 'x=' gives '', quoting is honoured"""
from http.cookiejar import split_header_words

# Each header is split on ',' into groups, each group a list of (name, value)
# pairs. A bare token has value None; "x=" gives ""; quoting is honoured.
SPLIT_CASES = [
    ("foo", [[("foo", None)]]),
    ("foo=bar", [[("foo", "bar")]]),
    ("   foo   ", [[("foo", None)]]),
    ("   foo=   ", [[("foo", "")]]),
    ("   foo=   ; bar= baz ", [[("foo", ""), ("bar", "baz")]]),
    ("foo=bar bar=baz", [[("foo", "bar"), ("bar", "baz")]]),
    ("foo= bar=baz", [[("foo", "bar=baz")]]),
    ("foo=bar;bar=baz", [[("foo", "bar"), ("bar", "baz")]]),
    ("foo bar baz", [[("foo", None), ("bar", None), ("baz", None)]]),
    ("a, b, c", [[("a", None)], [("b", None)], [("c", None)]]),
    (
        'foo; bar=baz, spam=, foo="\\,\\;\\"", bar= ',
        [
            [("foo", None), ("bar", "baz")],
            [("spam", "")],
            [("foo", ',;"')],
            [("bar", "")],
        ],
    ),
]
for arg, expect in SPLIT_CASES:
    assert split_header_words([arg]) == expect, arg

print("split_header_words_tokenizes OK")
