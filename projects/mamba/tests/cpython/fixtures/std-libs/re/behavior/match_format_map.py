# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "match_format_map"
# subject = "re.Match.__getitem__"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.Match.__getitem__: a Match object is usable as the mapping in str.format_map; unmatched named groups render as 'None'"""
import re

pat = re.compile(r"(?:(?P<a1>a)|(?P<b2>b))(?P<c3>c)?")
m = pat.match("a")
assert "{a1}/{b2}/{c3}".format_map(m) == "a/None/None", "format_map over match"

print("match_format_map OK")
