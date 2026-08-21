# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "urlencode_empty_and_coercion"
# subject = "urllib.parse.urlencode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.urlencode: empty mapping/sequence produce ''; non-string scalar values (int, None) are str()-coerced"""
from urllib.parse import urlencode

assert urlencode({}) == "", "empty dict"
assert urlencode([]) == "", "empty list"
assert urlencode({"a": 1}) == "a=1", "int value"
assert urlencode({"a": None}) == "a=None", "None value"

print("urlencode_empty_and_coercion OK")
