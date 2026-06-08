# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_getmembers_is_present"
# subject = "inspect.getmembers"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.getmembers: api_getmembers_is_present (surface)."""
import inspect

assert hasattr(inspect, "getmembers")
print("api_getmembers_is_present OK")
