# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "localtime_is_callable"
# subject = "time.localtime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""time.localtime: localtime_is_callable (surface)."""
import time

assert callable(time.localtime)
print("localtime_is_callable OK")
