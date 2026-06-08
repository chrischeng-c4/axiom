# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_callable_is_present"
# subject = "builtins.callable"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.callable: api_callable_is_present (surface)."""
import builtins

assert hasattr(builtins, "callable")
print("api_callable_is_present OK")
