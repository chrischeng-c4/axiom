# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_error"
# dimension = "surface"
# case = "api_url_error_is_present"
# subject = "urllib.error.URLError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.error.URLError: api_url_error_is_present (surface)."""
import urllib.error

assert hasattr(urllib.error, "URLError")
print("api_url_error_is_present OK")
