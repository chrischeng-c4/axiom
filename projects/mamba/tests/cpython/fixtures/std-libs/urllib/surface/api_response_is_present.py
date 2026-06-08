# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "surface"
# case = "api_response_is_present"
# subject = "urllib.response"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.response: api_response_is_present (surface)."""
import urllib.response

assert hasattr(urllib, "response")
print("api_response_is_present OK")
