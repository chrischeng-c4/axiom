# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "api_verbose_is_present"
# subject = "re.VERBOSE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""re.VERBOSE: api_verbose_is_present (surface)."""
import re

assert hasattr(re, "VERBOSE")
print("api_verbose_is_present OK")
