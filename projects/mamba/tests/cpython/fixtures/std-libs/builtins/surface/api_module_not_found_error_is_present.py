# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_module_not_found_error_is_present"
# subject = "builtins.ModuleNotFoundError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.ModuleNotFoundError: api_module_not_found_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "ModuleNotFoundError")
print("api_module_not_found_error_is_present OK")
