# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "api_multiline_is_present"
# subject = "re.MULTILINE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""re.MULTILINE: api_multiline_is_present (surface)."""
import re

assert hasattr(re, "MULTILINE")
print("api_multiline_is_present OK")
