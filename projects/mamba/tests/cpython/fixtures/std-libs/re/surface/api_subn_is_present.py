# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "api_subn_is_present"
# subject = "re.subn"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""re.subn: api_subn_is_present (surface)."""
import re

assert hasattr(re, "subn")
print("api_subn_is_present OK")
