# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "sub_replaces_all_matches"
# subject = "re.sub"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.sub: sub replaces every non-overlapping match by default: r'\\d+' -> 'NUM' on 'abc123def456' is 'abcNUMdefNUM'"""
import re

assert re.sub(r"\d+", "NUM", "abc123def456") == "abcNUMdefNUM"

print("sub_replaces_all_matches OK")
