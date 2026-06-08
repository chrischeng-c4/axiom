# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "api_match_is_present_2"
# subject = "re.match"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""re.match: api_match_is_present_2 (surface)."""
import re

assert hasattr(re, "match")
print("api_match_is_present_2 OK")
