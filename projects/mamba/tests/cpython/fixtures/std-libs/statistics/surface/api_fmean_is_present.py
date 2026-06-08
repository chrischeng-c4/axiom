# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "api_fmean_is_present"
# subject = "statistics.fmean"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""statistics.fmean: api_fmean_is_present (surface)."""
import statistics

assert hasattr(statistics, "fmean")
print("api_fmean_is_present OK")
