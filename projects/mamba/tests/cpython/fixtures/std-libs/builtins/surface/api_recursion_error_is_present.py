# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_recursion_error_is_present"
# subject = "builtins.RecursionError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.RecursionError: api_recursion_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "RecursionError")
print("api_recursion_error_is_present OK")
