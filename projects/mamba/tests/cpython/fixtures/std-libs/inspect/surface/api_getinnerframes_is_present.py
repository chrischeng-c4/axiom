# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_getinnerframes_is_present"
# subject = "inspect.getinnerframes"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.getinnerframes: api_getinnerframes_is_present (surface)."""
import inspect

assert hasattr(inspect, "getinnerframes")
print("api_getinnerframes_is_present OK")
