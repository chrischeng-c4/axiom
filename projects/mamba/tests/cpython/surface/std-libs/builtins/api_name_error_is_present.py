# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_name_error_is_present"
# subject = "builtins.NameError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.NameError: api_name_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "NameError")
print("api_name_error_is_present OK")
