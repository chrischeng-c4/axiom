# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "pstdev_is_callable"
# subject = "statistics.pstdev"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""statistics.pstdev: pstdev_is_callable (surface)."""
import statistics

assert callable(statistics.pstdev)
print("pstdev_is_callable OK")
