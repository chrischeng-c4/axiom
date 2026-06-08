# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "timeit"
# dimension = "surface"
# case = "api_repeat_is_present"
# subject = "timeit.repeat"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""timeit.repeat: api_repeat_is_present (surface)."""
import timeit

assert hasattr(timeit, "repeat")
print("api_repeat_is_present OK")
