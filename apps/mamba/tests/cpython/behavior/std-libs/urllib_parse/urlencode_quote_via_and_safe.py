# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "urlencode_quote_via_and_safe"
# subject = "urllib.parse.urlencode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlencode: urlencode defaults to quote_plus (space->'+') but quote_via=quote switches to %20, safe= passes named chars through, and non-str values are str()-coerced (including a custom __str__)"""
from urllib.parse import urlencode, quote

assert urlencode({"a": "some value"}) == "a=some+value"
assert urlencode({"a": "some value/another"}, quote_via=quote) == "a=some%20value%2Fanother"
assert urlencode({"a": "some value/another"}, safe="/", quote_via=quote) == "a=some%20value/another"


class _Trivial:
    def __str__(self):
        return "trivial"


assert urlencode({"a": _Trivial()}) == "a=trivial"

print("urlencode_quote_via_and_safe OK")
