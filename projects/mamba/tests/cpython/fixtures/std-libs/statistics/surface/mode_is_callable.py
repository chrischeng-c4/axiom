# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "mode_is_callable"
# subject = "statistics.mode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""statistics.mode: mode_is_callable (surface)."""
import statistics

assert callable(statistics.mode)
print("mode_is_callable OK")
