# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "surface"
# case = "api_fancy_ur_lopener_is_present"
# subject = "urllib.request.FancyURLopener"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.request.FancyURLopener: api_fancy_ur_lopener_is_present (surface)."""
import urllib.request

assert hasattr(urllib.request, "FancyURLopener")
print("api_fancy_ur_lopener_is_present OK")
