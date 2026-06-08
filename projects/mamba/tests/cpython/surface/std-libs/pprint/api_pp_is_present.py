# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "surface"
# case = "api_pp_is_present"
# subject = "pprint.pp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pprint.pp: api_pp_is_present (surface)."""
import pprint

assert hasattr(pprint, "pp")
print("api_pp_is_present OK")
