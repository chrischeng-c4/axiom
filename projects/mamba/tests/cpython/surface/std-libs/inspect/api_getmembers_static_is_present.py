# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_getmembers_static_is_present"
# subject = "inspect.getmembers_static"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.getmembers_static: api_getmembers_static_is_present (surface)."""
import inspect

assert hasattr(inspect, "getmembers_static")
print("api_getmembers_static_is_present OK")
