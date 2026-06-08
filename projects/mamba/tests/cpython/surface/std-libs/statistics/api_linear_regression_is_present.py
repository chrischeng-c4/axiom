# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "api_linear_regression_is_present"
# subject = "statistics.linear_regression"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""statistics.linear_regression: api_linear_regression_is_present (surface)."""
import statistics

assert hasattr(statistics, "linear_regression")
print("api_linear_regression_is_present OK")
