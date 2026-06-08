# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "perf_counter_is_callable"
# subject = "time.perf_counter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""time.perf_counter: perf_counter_is_callable (surface)."""
import time

assert callable(time.perf_counter)
print("perf_counter_is_callable OK")
