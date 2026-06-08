# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "group_multi_id_returns_tuple"
# subject = "re.Match.group"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.Match.group: group() with several ids returns them in id order: re.match(r'(a)(b)','ab').group(2,1) is ('b','a'), and group()==group(0)=='ab'"""
import re

m = re.match(r"(a)(b)", "ab")
assert m.group(2, 1) == ("b", "a"), "multi-id tuple in id order"
assert m.group() == "ab" and m.group(0) == "ab", "group() == group(0)"

print("group_multi_id_returns_tuple OK")
