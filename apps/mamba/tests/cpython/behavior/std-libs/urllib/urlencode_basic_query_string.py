# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "urlencode_basic_query_string"
# subject = "urllib.parse.urlencode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.urlencode: urlencode renders a mapping and an ordered list of pairs into an &-joined key=value query string, preserving list order"""
from urllib.parse import urlencode

enc = urlencode({"a": "1", "b": "2"})
assert "a=1" in enc and "b=2" in enc, repr(enc)
assert urlencode([("z", "1"), ("a", "2"), ("m", "3")]) == "z=1&a=2&m=3", "ordered"

print("urlencode_basic_query_string OK")
