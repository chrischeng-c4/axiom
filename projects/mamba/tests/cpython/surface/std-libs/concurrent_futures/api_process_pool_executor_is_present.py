# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "surface"
# case = "api_process_pool_executor_is_present"
# subject = "concurrent.futures.ProcessPoolExecutor"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""concurrent.futures.ProcessPoolExecutor: api_process_pool_executor_is_present (surface)."""
import concurrent.futures

assert hasattr(concurrent.futures, "ProcessPoolExecutor")
print("api_process_pool_executor_is_present OK")
