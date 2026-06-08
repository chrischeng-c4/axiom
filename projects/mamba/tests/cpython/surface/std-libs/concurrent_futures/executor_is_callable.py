# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "surface"
# case = "executor_is_callable"
# subject = "concurrent.futures.Executor"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.Executor: executor_is_callable (surface)."""
import concurrent.futures

assert callable(concurrent.futures.Executor)
print("executor_is_callable OK")
