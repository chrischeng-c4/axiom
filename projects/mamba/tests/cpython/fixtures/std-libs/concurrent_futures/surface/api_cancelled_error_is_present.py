# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "surface"
# case = "api_cancelled_error_is_present"
# subject = "concurrent.futures.CancelledError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""concurrent.futures.CancelledError: api_cancelled_error_is_present (surface)."""
import concurrent.futures

assert hasattr(concurrent.futures, "CancelledError")
print("api_cancelled_error_is_present OK")
