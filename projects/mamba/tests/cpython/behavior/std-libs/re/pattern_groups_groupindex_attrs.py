# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "pattern_groups_groupindex_attrs"
# subject = "re.Pattern.groupindex"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.Pattern.groupindex: a compiled Pattern reports .groups count, .groupindex name->number map, and the original .pattern source string"""
import re

cp = re.compile(r"(?P<first>a)(?P<other>b)", re.I)
assert cp.groups == 2, "pattern.groups count"
assert cp.groupindex == {"first": 1, "other": 2}, "pattern.groupindex map"
assert cp.pattern == r"(?P<first>a)(?P<other>b)", "pattern.pattern source"

print("pattern_groups_groupindex_attrs OK")
