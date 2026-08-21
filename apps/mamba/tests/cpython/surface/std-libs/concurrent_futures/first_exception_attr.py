# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "surface"
# case = "first_exception_attr"
# subject = "concurrent.futures"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures: first_exception_attr (surface)."""
import concurrent.futures

assert hasattr(concurrent.futures, "FIRST_EXCEPTION")
print("first_exception_attr OK")
