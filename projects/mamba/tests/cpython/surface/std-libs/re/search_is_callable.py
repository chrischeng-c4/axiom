# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "search_is_callable"
# subject = "re.search"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.search: search_is_callable (surface)."""
import re

assert callable(re.search)
print("search_is_callable OK")
