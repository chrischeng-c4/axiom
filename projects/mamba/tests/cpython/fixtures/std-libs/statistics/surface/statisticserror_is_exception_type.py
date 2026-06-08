# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "statisticserror_is_exception_type"
# subject = "statistics.StatisticsError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""statistics.StatisticsError: statisticserror_is_exception_type (surface)."""
import statistics

assert hasattr(statistics.StatisticsError, "__cause__")
print("statisticserror_is_exception_type OK")
