# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "surface"
# case = "api_getproxies_is_present"
# subject = "urllib.request.getproxies"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.request.getproxies: api_getproxies_is_present (surface)."""
import urllib.request

assert hasattr(urllib.request, "getproxies")
print("api_getproxies_is_present OK")
