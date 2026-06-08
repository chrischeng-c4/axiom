# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "api_purge_is_present"
# subject = "re.purge"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""re.purge: api_purge_is_present (surface)."""
import re

assert hasattr(re, "purge")
print("api_purge_is_present OK")
