# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_error"
# dimension = "surface"
# case = "api_http_error_is_present"
# subject = "urllib.error.HTTPError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.error.HTTPError: api_http_error_is_present (surface)."""
import urllib.error

assert hasattr(urllib.error, "HTTPError")
print("api_http_error_is_present OK")
