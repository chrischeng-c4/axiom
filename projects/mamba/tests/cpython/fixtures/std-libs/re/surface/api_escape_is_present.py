# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "api_escape_is_present"
# subject = "re.escape"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""re.escape: api_escape_is_present (surface)."""
import re

assert hasattr(re, "escape")
print("api_escape_is_present OK")
