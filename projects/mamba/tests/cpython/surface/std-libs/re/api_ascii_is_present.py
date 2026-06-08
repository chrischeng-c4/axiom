# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "api_ascii_is_present"
# subject = "re.ASCII"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""re.ASCII: api_ascii_is_present (surface)."""
import re

assert hasattr(re, "ASCII")
print("api_ascii_is_present OK")
