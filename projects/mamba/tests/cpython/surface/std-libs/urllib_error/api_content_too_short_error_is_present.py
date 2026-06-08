# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_error"
# dimension = "surface"
# case = "api_content_too_short_error_is_present"
# subject = "urllib.error.ContentTooShortError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.error.ContentTooShortError: api_content_too_short_error_is_present (surface)."""
import urllib.error

assert hasattr(urllib.error, "ContentTooShortError")
print("api_content_too_short_error_is_present OK")
