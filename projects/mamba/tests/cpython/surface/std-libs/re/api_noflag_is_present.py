# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "api_noflag_is_present"
# subject = "re.NOFLAG"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""re.NOFLAG: api_noflag_is_present (surface)."""
import re

assert hasattr(re, "NOFLAG")
print("api_noflag_is_present OK")
