# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "harmonic_mean_is_callable"
# subject = "statistics.harmonic_mean"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""statistics.harmonic_mean: harmonic_mean_is_callable (surface)."""
import statistics

assert callable(statistics.harmonic_mean)
print("harmonic_mean_is_callable OK")
