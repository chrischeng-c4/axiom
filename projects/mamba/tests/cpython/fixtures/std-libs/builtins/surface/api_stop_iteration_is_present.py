# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_stop_iteration_is_present"
# subject = "builtins.StopIteration"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.StopIteration: api_stop_iteration_is_present (surface)."""
import builtins

assert hasattr(builtins, "StopIteration")
print("api_stop_iteration_is_present OK")
