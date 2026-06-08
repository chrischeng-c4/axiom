# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "surface"
# case = "api_isrecursive_is_present"
# subject = "pprint.isrecursive"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pprint.isrecursive: api_isrecursive_is_present (surface)."""
import pprint

assert hasattr(pprint, "isrecursive")
print("api_isrecursive_is_present OK")
