# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_getmro_is_present"
# subject = "inspect.getmro"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.getmro: api_getmro_is_present (surface)."""
import inspect

assert hasattr(inspect, "getmro")
print("api_getmro_is_present OK")
