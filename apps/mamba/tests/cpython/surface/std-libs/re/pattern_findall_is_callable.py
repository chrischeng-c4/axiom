# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "pattern_findall_is_callable"
# subject = "re.Pattern.findall"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.Pattern.findall: pattern_findall_is_callable (surface)."""
import re

assert callable(re.Pattern.findall)
print("pattern_findall_is_callable OK")
