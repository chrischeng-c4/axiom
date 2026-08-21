# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "pattern_search_is_callable"
# subject = "re.Pattern.search"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.Pattern.search: pattern_search_is_callable (surface)."""
import re

assert callable(re.Pattern.search)
print("pattern_search_is_callable OK")
