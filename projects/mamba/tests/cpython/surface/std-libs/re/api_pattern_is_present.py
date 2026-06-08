# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "api_pattern_is_present"
# subject = "re.Pattern"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""re.Pattern: api_pattern_is_present (surface)."""
import re

assert hasattr(re, "Pattern")
print("api_pattern_is_present OK")
