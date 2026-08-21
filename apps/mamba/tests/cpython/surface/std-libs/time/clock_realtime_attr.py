# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "clock_realtime_attr"
# subject = "time"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""time: clock_realtime_attr (surface)."""
import time

assert hasattr(time, "CLOCK_REALTIME")
print("clock_realtime_attr OK")
