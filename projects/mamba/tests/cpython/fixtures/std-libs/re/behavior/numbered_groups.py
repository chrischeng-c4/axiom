# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "numbered_groups"
# subject = "re.Match.group"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.Match.group: numbered groups: re.search(r'(\\w+)\\s+(\\w+)','hello world') gives group(1)='hello', group(2)='world', groups()=('hello','world')"""
import re

m = re.search(r"(\w+)\s+(\w+)", "hello world")
assert m is not None, "groups match"
assert m.group(1) == "hello", f"g1 = {m.group(1)!r}"
assert m.group(2) == "world", f"g2 = {m.group(2)!r}"
assert m.groups() == ("hello", "world"), f"groups() = {m.groups()!r}"

print("numbered_groups OK")
