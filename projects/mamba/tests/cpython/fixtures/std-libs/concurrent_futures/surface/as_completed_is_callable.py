# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "surface"
# case = "as_completed_is_callable"
# subject = "concurrent.futures.as_completed"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.as_completed: as_completed_is_callable (surface)."""
import concurrent.futures

assert callable(concurrent.futures.as_completed)
print("as_completed_is_callable OK")
