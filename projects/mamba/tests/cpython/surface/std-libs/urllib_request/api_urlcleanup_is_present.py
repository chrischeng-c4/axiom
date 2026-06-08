# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "surface"
# case = "api_urlcleanup_is_present"
# subject = "urllib.request.urlcleanup"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.request.urlcleanup: api_urlcleanup_is_present (surface)."""
import urllib.request

assert hasattr(urllib.request, "urlcleanup")
print("api_urlcleanup_is_present OK")
