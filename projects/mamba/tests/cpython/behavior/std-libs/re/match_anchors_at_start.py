# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "match_anchors_at_start"
# subject = "re.match"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.match: re.match anchors at the start: r'\\d+' matches '123abc' (group '123') but returns None for 'abc123'"""
import re

m = re.match(r"\d+", "123abc")
assert m is not None, "match at start"
assert m.group() == "123", f"group = {m.group()!r}"
assert re.match(r"\d+", "abc123") is None, "no match when not at start"

print("match_anchors_at_start OK")
