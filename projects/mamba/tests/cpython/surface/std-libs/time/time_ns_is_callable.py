# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "time_ns_is_callable"
# subject = "time.time_ns"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""time.time_ns: time_ns_is_callable (surface)."""
import time

assert callable(time.time_ns)
print("time_ns_is_callable OK")
