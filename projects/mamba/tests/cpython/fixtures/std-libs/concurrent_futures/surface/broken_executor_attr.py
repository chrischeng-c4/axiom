# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "surface"
# case = "broken_executor_attr"
# subject = "concurrent.futures"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures: broken_executor_attr (surface)."""
import concurrent.futures

assert hasattr(concurrent.futures, "BrokenExecutor")
print("broken_executor_attr OK")
