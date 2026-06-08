# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "api_sub_is_present"
# subject = "re.sub"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""re.sub: api_sub_is_present (surface)."""
import re

assert hasattr(re, "sub")
print("api_sub_is_present OK")
