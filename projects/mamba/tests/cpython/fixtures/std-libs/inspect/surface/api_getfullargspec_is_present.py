# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_getfullargspec_is_present"
# subject = "inspect.getfullargspec"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.getfullargspec: api_getfullargspec_is_present (surface)."""
import inspect

assert hasattr(inspect, "getfullargspec")
print("api_getfullargspec_is_present OK")
