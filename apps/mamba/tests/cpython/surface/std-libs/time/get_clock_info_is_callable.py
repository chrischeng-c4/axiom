# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "get_clock_info_is_callable"
# subject = "time.get_clock_info"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""time.get_clock_info: get_clock_info_is_callable (surface)."""
import time

assert callable(time.get_clock_info)
print("get_clock_info_is_callable OK")
