# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "surface"
# case = "api_all_completed_is_present"
# subject = "concurrent.futures.ALL_COMPLETED"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""concurrent.futures.ALL_COMPLETED: api_all_completed_is_present (surface)."""
import concurrent.futures

assert hasattr(concurrent.futures, "ALL_COMPLETED")
print("api_all_completed_is_present OK")
