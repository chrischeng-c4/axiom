# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_full_arg_spec_is_present"
# subject = "inspect.FullArgSpec"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.FullArgSpec: api_full_arg_spec_is_present (surface)."""
import inspect

assert hasattr(inspect, "FullArgSpec")
print("api_full_arg_spec_is_present OK")
