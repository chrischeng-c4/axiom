# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "surface"
# case = "process_pool_executor_is_callable"
# subject = "concurrent.futures.ProcessPoolExecutor"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.ProcessPoolExecutor: process_pool_executor_is_callable (surface)."""
import concurrent.futures

assert callable(concurrent.futures.ProcessPoolExecutor)
print("process_pool_executor_is_callable OK")
