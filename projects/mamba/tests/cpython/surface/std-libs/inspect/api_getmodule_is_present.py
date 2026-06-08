# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_getmodule_is_present"
# subject = "inspect.getmodule"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.getmodule: api_getmodule_is_present (surface)."""
import inspect

assert hasattr(inspect, "getmodule")
print("api_getmodule_is_present OK")
