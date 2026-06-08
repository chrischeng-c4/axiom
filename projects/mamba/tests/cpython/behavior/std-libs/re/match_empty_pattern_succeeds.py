# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "match_empty_pattern_succeeds"
# subject = "re.match"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.match: the empty pattern matches at the start of any string: re.match('', 'abc') is not None"""
import re

m = re.match(r"", "abc")
assert m is not None, "empty pattern matches at start"
assert m.group() == "", f"empty match group = {m.group()!r}"

print("match_empty_pattern_succeeds OK")
