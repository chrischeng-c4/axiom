# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "api_fullmatch_is_present"
# subject = "re.fullmatch"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""re.fullmatch: api_fullmatch_is_present (surface)."""
import re

assert hasattr(re, "fullmatch")
print("api_fullmatch_is_present OK")
