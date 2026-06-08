# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "surface"
# case = "api_ur_lopener_is_present"
# subject = "urllib.request.URLopener"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.request.URLopener: api_ur_lopener_is_present (surface)."""
import urllib.request

assert hasattr(urllib.request, "URLopener")
print("api_ur_lopener_is_present OK")
