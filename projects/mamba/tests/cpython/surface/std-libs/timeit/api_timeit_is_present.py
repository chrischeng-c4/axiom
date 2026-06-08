# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "timeit"
# dimension = "surface"
# case = "api_timeit_is_present"
# subject = "timeit.timeit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""timeit.timeit: api_timeit_is_present (surface)."""
import timeit

assert hasattr(timeit, "timeit")
print("api_timeit_is_present OK")
