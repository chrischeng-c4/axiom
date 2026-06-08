# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "api_harmonic_mean_is_present"
# subject = "statistics.harmonic_mean"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""statistics.harmonic_mean: api_harmonic_mean_is_present (surface)."""
import statistics

assert hasattr(statistics, "harmonic_mean")
print("api_harmonic_mean_is_present OK")
