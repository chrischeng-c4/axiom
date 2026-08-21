# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "match_group_is_callable"
# subject = "re.Match.group"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.Match.group: match_group_is_callable (surface)."""
import re

assert callable(re.Match.group)
print("match_group_is_callable OK")
