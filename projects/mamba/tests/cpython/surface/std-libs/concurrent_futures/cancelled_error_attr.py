# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "surface"
# case = "cancelled_error_attr"
# subject = "concurrent.futures"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures: cancelled_error_attr (surface)."""
import concurrent.futures

assert hasattr(concurrent.futures, "CancelledError")
print("cancelled_error_attr OK")
