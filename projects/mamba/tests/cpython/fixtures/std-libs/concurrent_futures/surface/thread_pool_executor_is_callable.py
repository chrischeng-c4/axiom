# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "surface"
# case = "thread_pool_executor_is_callable"
# subject = "concurrent.futures.ThreadPoolExecutor"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.ThreadPoolExecutor: thread_pool_executor_is_callable (surface)."""
import concurrent.futures

assert callable(concurrent.futures.ThreadPoolExecutor)
print("thread_pool_executor_is_callable OK")
