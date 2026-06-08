# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_arg_info_is_present"
# subject = "inspect.ArgInfo"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.ArgInfo: api_arg_info_is_present (surface)."""
import inspect

assert hasattr(inspect, "ArgInfo")
print("api_arg_info_is_present OK")
