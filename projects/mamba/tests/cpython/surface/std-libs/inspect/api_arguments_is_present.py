# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_arguments_is_present"
# subject = "inspect.Arguments"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.Arguments: api_arguments_is_present (surface)."""
import inspect

assert hasattr(inspect, "Arguments")
print("api_arguments_is_present OK")
