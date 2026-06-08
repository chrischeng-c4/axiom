# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "surface"
# case = "reset_peak_is_callable"
# subject = "tracemalloc.reset_peak"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tracemalloc.reset_peak: reset_peak_is_callable (surface)."""
import tracemalloc

assert callable(tracemalloc.reset_peak)
print("reset_peak_is_callable OK")
