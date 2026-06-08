# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "surface"
# case = "api_pprint_is_present"
# subject = "pprint.pprint"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pprint.pprint: api_pprint_is_present (surface)."""
import pprint

assert hasattr(pprint, "pprint")
print("api_pprint_is_present OK")
