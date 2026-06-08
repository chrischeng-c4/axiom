# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_closure_vars_is_present"
# subject = "inspect.ClosureVars"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.ClosureVars: api_closure_vars_is_present (surface)."""
import inspect

assert hasattr(inspect, "ClosureVars")
print("api_closure_vars_is_present OK")
