# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "process_time_is_callable"
# subject = "time.process_time"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""time.process_time: process_time_is_callable (surface)."""
import time

assert callable(time.process_time)
print("process_time_is_callable OK")
