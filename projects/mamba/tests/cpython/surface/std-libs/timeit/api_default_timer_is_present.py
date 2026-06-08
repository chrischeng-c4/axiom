# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "timeit"
# dimension = "surface"
# case = "api_default_timer_is_present"
# subject = "timeit.default_timer"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""timeit.default_timer: api_default_timer_is_present (surface)."""
import timeit

assert hasattr(timeit, "default_timer")
print("api_default_timer_is_present OK")
