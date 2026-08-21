# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "match_is_callable"
# subject = "re.match"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.match: match_is_callable (surface)."""
import re

assert callable(re.match)
print("match_is_callable OK")
