# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_getmodulename_is_present"
# subject = "inspect.getmodulename"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.getmodulename: api_getmodulename_is_present (surface)."""
import inspect

assert hasattr(inspect, "getmodulename")
print("api_getmodulename_is_present OK")
