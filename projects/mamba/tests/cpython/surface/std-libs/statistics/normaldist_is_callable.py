# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "normaldist_is_callable"
# subject = "statistics.NormalDist"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""statistics.NormalDist: normaldist_is_callable (surface)."""
import statistics

assert callable(statistics.NormalDist)
print("normaldist_is_callable OK")
