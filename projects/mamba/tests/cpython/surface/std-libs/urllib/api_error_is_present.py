# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "surface"
# case = "api_error_is_present"
# subject = "urllib.error"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.error: api_error_is_present (surface)."""
import urllib.error

assert hasattr(urllib, "error")
print("api_error_is_present OK")
