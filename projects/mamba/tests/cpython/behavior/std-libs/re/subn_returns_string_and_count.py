# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "subn_returns_string_and_count"
# subject = "re.subn"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.subn: subn returns (new_string, num_subs): re.subn(r'\\d+','X','a1 b22 c333') is ('aX bX cX', 3); a no-match leaves count 0"""
import re

assert re.subn(r"\d+", "X", "a1 b22 c333") == ("aX bX cX", 3), "subn count"
assert re.subn(r"b+", "x", "xyz") == ("xyz", 0), "subn no match -> count 0"

print("subn_returns_string_and_count OK")
