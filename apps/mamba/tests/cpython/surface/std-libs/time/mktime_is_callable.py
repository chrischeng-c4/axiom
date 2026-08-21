# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "mktime_is_callable"
# subject = "time.mktime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""time.mktime: mktime_is_callable (surface)."""
import time

assert callable(time.mktime)
print("mktime_is_callable OK")
