# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_response"
# dimension = "surface"
# case = "api_addinfo_is_present"
# subject = "urllib.response.addinfo"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.response.addinfo: api_addinfo_is_present (surface)."""
import urllib.response

assert hasattr(urllib.response, "addinfo")
print("api_addinfo_is_present OK")
