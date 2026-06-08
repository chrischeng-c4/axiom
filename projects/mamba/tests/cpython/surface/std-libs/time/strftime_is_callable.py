# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "strftime_is_callable"
# subject = "time.strftime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""time.strftime: strftime_is_callable (surface)."""
import time

assert callable(time.strftime)
print("strftime_is_callable OK")
