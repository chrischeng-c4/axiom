# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_bound_arguments_is_present"
# subject = "inspect.BoundArguments"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.BoundArguments: api_bound_arguments_is_present (surface)."""
import inspect

assert hasattr(inspect, "BoundArguments")
print("api_bound_arguments_is_present OK")
