# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_getlineno_is_present"
# subject = "inspect.getlineno"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.getlineno: api_getlineno_is_present (surface)."""
import inspect

assert hasattr(inspect, "getlineno")
print("api_getlineno_is_present OK")
