# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "sub_count_limits_replacements"
# subject = "re.sub"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.sub: the count keyword caps replacements: re.sub(r'a','b','aaaaa',count=1) is 'baaaa' while count=0 means replace all"""
import re

assert re.sub(r"a", "b", "aaaaa") == "bbbbb", "count default -> all"
assert re.sub(r"a", "b", "aaaaa", count=0) == "bbbbb", "count=0 -> all"
assert re.sub(r"a", "b", "aaaaa", count=1) == "baaaa", "count=1 -> one"

print("sub_count_limits_replacements OK")
