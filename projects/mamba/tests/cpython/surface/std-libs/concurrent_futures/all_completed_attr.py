# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "surface"
# case = "all_completed_attr"
# subject = "concurrent.futures"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures: all_completed_attr (surface)."""
import concurrent.futures

assert hasattr(concurrent.futures, "ALL_COMPLETED")
print("all_completed_attr OK")
