# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_parameter_is_present"
# subject = "inspect.Parameter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.Parameter: api_parameter_is_present (surface)."""
import inspect

assert hasattr(inspect, "Parameter")
print("api_parameter_is_present OK")
