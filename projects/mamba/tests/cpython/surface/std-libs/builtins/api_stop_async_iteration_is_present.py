# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_stop_async_iteration_is_present"
# subject = "builtins.StopAsyncIteration"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.StopAsyncIteration: api_stop_async_iteration_is_present (surface)."""
import builtins

assert hasattr(builtins, "StopAsyncIteration")
print("api_stop_async_iteration_is_present OK")
