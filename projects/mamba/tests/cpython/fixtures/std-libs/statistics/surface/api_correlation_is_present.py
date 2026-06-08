# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "api_correlation_is_present"
# subject = "statistics.correlation"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""statistics.correlation: api_correlation_is_present (surface)."""
import statistics

assert hasattr(statistics, "correlation")
print("api_correlation_is_present OK")
