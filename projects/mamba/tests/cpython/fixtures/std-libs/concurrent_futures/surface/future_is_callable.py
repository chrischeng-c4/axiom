# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "surface"
# case = "future_is_callable"
# subject = "concurrent.futures.Future"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.Future: future_is_callable (surface)."""
import concurrent.futures

assert callable(concurrent.futures.Future)
print("future_is_callable OK")
