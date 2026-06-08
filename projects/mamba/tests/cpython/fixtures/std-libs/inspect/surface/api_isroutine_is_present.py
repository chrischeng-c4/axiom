# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_isroutine_is_present"
# subject = "inspect.isroutine"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.isroutine: api_isroutine_is_present (surface)."""
import inspect

assert hasattr(inspect, "isroutine")
print("api_isroutine_is_present OK")
