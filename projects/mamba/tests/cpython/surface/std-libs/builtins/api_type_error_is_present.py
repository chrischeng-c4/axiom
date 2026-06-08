# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_type_error_is_present"
# subject = "builtins.TypeError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.TypeError: api_type_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "TypeError")
print("api_type_error_is_present OK")
