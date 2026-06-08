# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "api_split_is_present"
# subject = "re.split"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""re.split: api_split_is_present (surface)."""
import re

assert hasattr(re, "split")
print("api_split_is_present OK")
