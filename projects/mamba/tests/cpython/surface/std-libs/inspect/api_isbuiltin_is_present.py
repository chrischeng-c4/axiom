# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_isbuiltin_is_present"
# subject = "inspect.isbuiltin"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.isbuiltin: api_isbuiltin_is_present (surface)."""
import inspect

assert hasattr(inspect, "isbuiltin")
print("api_isbuiltin_is_present OK")
