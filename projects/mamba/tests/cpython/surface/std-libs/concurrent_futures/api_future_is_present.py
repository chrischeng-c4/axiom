# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "surface"
# case = "api_future_is_present"
# subject = "concurrent.futures.Future"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""concurrent.futures.Future: api_future_is_present (surface)."""
import concurrent.futures

assert hasattr(concurrent.futures, "Future")
print("api_future_is_present OK")
