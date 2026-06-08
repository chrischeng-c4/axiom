# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "api_unicode_is_present"
# subject = "re.UNICODE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""re.UNICODE: api_unicode_is_present (surface)."""
import re

assert hasattr(re, "UNICODE")
print("api_unicode_is_present OK")
