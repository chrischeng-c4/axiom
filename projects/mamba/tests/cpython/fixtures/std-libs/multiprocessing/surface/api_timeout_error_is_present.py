# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "api_timeout_error_is_present"
# subject = "multiprocessing.TimeoutError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""multiprocessing.TimeoutError: api_timeout_error_is_present (surface)."""
import multiprocessing

assert hasattr(multiprocessing, "TimeoutError")
print("api_timeout_error_is_present OK")
