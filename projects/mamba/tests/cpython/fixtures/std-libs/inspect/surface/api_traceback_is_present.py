# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_traceback_is_present"
# subject = "inspect.Traceback"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.Traceback: api_traceback_is_present (surface)."""
import inspect

assert hasattr(inspect, "Traceback")
print("api_traceback_is_present OK")
