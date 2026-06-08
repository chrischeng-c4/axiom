# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_ismethod_is_present"
# subject = "inspect.ismethod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.ismethod: api_ismethod_is_present (surface)."""
import inspect

assert hasattr(inspect, "ismethod")
print("api_ismethod_is_present OK")
