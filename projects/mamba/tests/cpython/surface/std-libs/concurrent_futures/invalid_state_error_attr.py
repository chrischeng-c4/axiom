# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "surface"
# case = "invalid_state_error_attr"
# subject = "concurrent.futures"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures: invalid_state_error_attr (surface)."""
import concurrent.futures

assert hasattr(concurrent.futures, "InvalidStateError")
print("invalid_state_error_attr OK")
