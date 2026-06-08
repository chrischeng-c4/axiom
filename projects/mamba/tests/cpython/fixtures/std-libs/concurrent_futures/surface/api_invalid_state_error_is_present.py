# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "surface"
# case = "api_invalid_state_error_is_present"
# subject = "concurrent.futures.InvalidStateError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""concurrent.futures.InvalidStateError: api_invalid_state_error_is_present (surface)."""
import concurrent.futures

assert hasattr(concurrent.futures, "InvalidStateError")
print("api_invalid_state_error_is_present OK")
