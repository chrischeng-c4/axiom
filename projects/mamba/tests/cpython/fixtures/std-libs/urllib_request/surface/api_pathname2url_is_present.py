# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "surface"
# case = "api_pathname2url_is_present"
# subject = "urllib.request.pathname2url"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.request.pathname2url: api_pathname2url_is_present (surface)."""
import urllib.request

assert hasattr(urllib.request, "pathname2url")
print("api_pathname2url_is_present OK")
