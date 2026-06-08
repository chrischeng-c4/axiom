# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_isframe_is_present"
# subject = "inspect.isframe"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.isframe: api_isframe_is_present (surface)."""
import inspect

assert hasattr(inspect, "isframe")
print("api_isframe_is_present OK")
