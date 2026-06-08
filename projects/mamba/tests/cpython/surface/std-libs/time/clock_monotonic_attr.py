# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "clock_monotonic_attr"
# subject = "time"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""time: clock_monotonic_attr (surface)."""
import time

assert hasattr(time, "CLOCK_MONOTONIC")
print("clock_monotonic_attr OK")
