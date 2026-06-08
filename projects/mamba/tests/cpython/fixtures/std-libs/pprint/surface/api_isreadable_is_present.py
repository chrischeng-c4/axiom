# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "surface"
# case = "api_isreadable_is_present"
# subject = "pprint.isreadable"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pprint.isreadable: api_isreadable_is_present (surface)."""
import pprint

assert hasattr(pprint, "isreadable")
print("api_isreadable_is_present OK")
