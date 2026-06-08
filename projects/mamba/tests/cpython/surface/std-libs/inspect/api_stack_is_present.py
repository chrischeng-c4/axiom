# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_stack_is_present"
# subject = "inspect.stack"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.stack: api_stack_is_present (surface)."""
import inspect

assert hasattr(inspect, "stack")
print("api_stack_is_present OK")
