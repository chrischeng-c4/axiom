# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "fullmatch_is_callable"
# subject = "re.fullmatch"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.fullmatch: fullmatch_is_callable (surface)."""
import re

assert callable(re.fullmatch)
print("fullmatch_is_callable OK")
