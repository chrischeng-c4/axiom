# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "api_ignorecase_is_present"
# subject = "re.IGNORECASE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""re.IGNORECASE: api_ignorecase_is_present (surface)."""
import re

assert hasattr(re, "IGNORECASE")
print("api_ignorecase_is_present OK")
