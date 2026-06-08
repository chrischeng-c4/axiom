# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "strptime_is_callable"
# subject = "time.strptime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""time.strptime: strptime_is_callable (surface)."""
import time

assert callable(time.strptime)
print("strptime_is_callable OK")
