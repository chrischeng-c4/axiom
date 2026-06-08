# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_getsourcelines_is_present"
# subject = "inspect.getsourcelines"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.getsourcelines: api_getsourcelines_is_present (surface)."""
import inspect

assert hasattr(inspect, "getsourcelines")
print("api_getsourcelines_is_present OK")
