# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "asctime_is_callable"
# subject = "time.asctime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""time.asctime: asctime_is_callable (surface)."""
import time

assert callable(time.asctime)
print("asctime_is_callable OK")
