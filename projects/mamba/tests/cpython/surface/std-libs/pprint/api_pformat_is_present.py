# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "surface"
# case = "api_pformat_is_present"
# subject = "pprint.pformat"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pprint.pformat: api_pformat_is_present (surface)."""
import pprint

assert hasattr(pprint, "pformat")
print("api_pformat_is_present OK")
