# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_isawaitable_is_present"
# subject = "inspect.isawaitable"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.isawaitable: api_isawaitable_is_present (surface)."""
import inspect

assert hasattr(inspect, "isawaitable")
print("api_isawaitable_is_present OK")
