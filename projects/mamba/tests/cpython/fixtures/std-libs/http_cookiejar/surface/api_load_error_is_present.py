# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "surface"
# case = "api_load_error_is_present"
# subject = "http.cookiejar.LoadError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""http.cookiejar.LoadError: api_load_error_is_present (surface)."""
import http.cookiejar

assert hasattr(http.cookiejar, "LoadError")
print("api_load_error_is_present OK")
