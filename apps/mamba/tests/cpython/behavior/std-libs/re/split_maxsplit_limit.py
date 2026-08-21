# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "split_maxsplit_limit"
# subject = "re.split"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.split: maxsplit caps the number of splits: re.split(r':','a:b:c:d',maxsplit=2) is ['a','b','c:d']"""
import re

assert re.split(r":", "a:b:c:d") == ["a", "b", "c", "d"], "no limit"
assert re.split(r":", "a:b:c:d", maxsplit=2) == ["a", "b", "c:d"], "maxsplit=2"

print("split_maxsplit_limit OK")
