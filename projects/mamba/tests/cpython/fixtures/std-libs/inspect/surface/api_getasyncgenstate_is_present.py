# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_getasyncgenstate_is_present"
# subject = "inspect.getasyncgenstate"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.getasyncgenstate: api_getasyncgenstate_is_present (surface)."""
import inspect

assert hasattr(inspect, "getasyncgenstate")
print("api_getasyncgenstate_is_present OK")
