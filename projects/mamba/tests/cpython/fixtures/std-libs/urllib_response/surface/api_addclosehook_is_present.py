# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_response"
# dimension = "surface"
# case = "api_addclosehook_is_present"
# subject = "urllib.response.addclosehook"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.response.addclosehook: api_addclosehook_is_present (surface)."""
import urllib.response

assert hasattr(urllib.response, "addclosehook")
print("api_addclosehook_is_present OK")
