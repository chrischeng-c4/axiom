# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "api_finditer_is_present"
# subject = "re.finditer"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""re.finditer: api_finditer_is_present (surface)."""
import re

assert hasattr(re, "finditer")
print("api_finditer_is_present OK")
