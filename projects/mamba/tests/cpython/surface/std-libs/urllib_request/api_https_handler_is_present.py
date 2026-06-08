# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "surface"
# case = "api_https_handler_is_present"
# subject = "urllib.request.HTTPSHandler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.request.HTTPSHandler: api_https_handler_is_present (surface)."""
import urllib.request

assert hasattr(urllib.request, "HTTPSHandler")
print("api_https_handler_is_present OK")
