# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "match_groups_is_callable"
# subject = "re.Match.groups"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.Match.groups: match_groups_is_callable (surface)."""
import re

assert callable(re.Match.groups)
print("match_groups_is_callable OK")
