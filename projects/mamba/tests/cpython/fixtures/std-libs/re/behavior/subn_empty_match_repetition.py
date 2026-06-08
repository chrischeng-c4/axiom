# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "subn_empty_match_repetition"
# subject = "re.subn"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.subn: an empty-matching pattern inserts between every character: re.subn(r'b*','x','xyz') is ('xxxyxzx', 4)"""
import re

assert re.subn(r"b*", "x", "xyz") == ("xxxyxzx", 4), "empty matches between chars"
assert re.subn(r"b*", "x", "xyz", count=2) == ("xxxyz", 2), "count limit on empty matches"

print("subn_empty_match_repetition OK")
