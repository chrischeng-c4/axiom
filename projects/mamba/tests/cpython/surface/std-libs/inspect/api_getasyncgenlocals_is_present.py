# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_getasyncgenlocals_is_present"
# subject = "inspect.getasyncgenlocals"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.getasyncgenlocals: api_getasyncgenlocals_is_present (surface)."""
import inspect

assert hasattr(inspect, "getasyncgenlocals")
print("api_getasyncgenlocals_is_present OK")
