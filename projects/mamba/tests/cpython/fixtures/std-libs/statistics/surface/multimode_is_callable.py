# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "multimode_is_callable"
# subject = "statistics.multimode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""statistics.multimode: multimode_is_callable (surface)."""
import statistics

assert callable(statistics.multimode)
print("multimode_is_callable OK")
