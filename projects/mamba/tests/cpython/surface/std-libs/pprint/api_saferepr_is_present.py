# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "surface"
# case = "api_saferepr_is_present"
# subject = "pprint.saferepr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pprint.saferepr: api_saferepr_is_present (surface)."""
import pprint

assert hasattr(pprint, "saferepr")
print("api_saferepr_is_present OK")
