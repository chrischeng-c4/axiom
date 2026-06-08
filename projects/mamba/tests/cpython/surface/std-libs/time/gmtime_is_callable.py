# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "gmtime_is_callable"
# subject = "time.gmtime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""time.gmtime: gmtime_is_callable (surface)."""
import time

assert callable(time.gmtime)
print("gmtime_is_callable OK")
