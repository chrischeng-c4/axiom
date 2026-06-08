# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "api_statistics_error_is_present"
# subject = "statistics.StatisticsError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""statistics.StatisticsError: api_statistics_error_is_present (surface)."""
import statistics

assert hasattr(statistics, "StatisticsError")
print("api_statistics_error_is_present OK")
