# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "escape_is_callable"
# subject = "re.escape"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.escape: escape_is_callable (surface)."""
import re

assert callable(re.escape)
print("escape_is_callable OK")
