# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_currentframe_is_present"
# subject = "inspect.currentframe"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.currentframe: api_currentframe_is_present (surface)."""
import inspect

assert hasattr(inspect, "currentframe")
print("api_currentframe_is_present OK")
