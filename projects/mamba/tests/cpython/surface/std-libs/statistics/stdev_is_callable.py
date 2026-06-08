# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "stdev_is_callable"
# subject = "statistics.stdev"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""statistics.stdev: stdev_is_callable (surface)."""
import statistics

assert callable(statistics.stdev)
print("stdev_is_callable OK")
