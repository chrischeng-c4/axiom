# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_getattr_static_is_present"
# subject = "inspect.getattr_static"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.getattr_static: api_getattr_static_is_present (surface)."""
import inspect

assert hasattr(inspect, "getattr_static")
print("api_getattr_static_is_present OK")
