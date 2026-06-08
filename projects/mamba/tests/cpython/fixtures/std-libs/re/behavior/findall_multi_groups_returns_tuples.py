# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "findall_multi_groups_returns_tuples"
# subject = "re.findall"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.findall: findall with multiple groups returns a list of group tuples: r'(\\w+)=(\\d+)' on 'a=1 b=2 c=3' is [('a','1'),('b','2'),('c','3')]"""
import re

matches = re.findall(r"(\w+)=(\d+)", "a=1 b=2 c=3")
assert matches == [("a", "1"), ("b", "2"), ("c", "3")], f"findall groups = {matches!r}"

print("findall_multi_groups_returns_tuples OK")
