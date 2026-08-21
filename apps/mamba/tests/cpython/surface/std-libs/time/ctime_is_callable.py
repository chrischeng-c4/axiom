# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "ctime_is_callable"
# subject = "time.ctime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""time.ctime: ctime_is_callable (surface)."""
import time

assert callable(time.ctime)
print("ctime_is_callable OK")
