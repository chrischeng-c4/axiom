# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "timeit"
# dimension = "surface"
# case = "api_timer_is_present"
# subject = "timeit.Timer"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""timeit.Timer: api_timer_is_present (surface)."""
import timeit

assert hasattr(timeit, "Timer")
print("api_timer_is_present OK")
