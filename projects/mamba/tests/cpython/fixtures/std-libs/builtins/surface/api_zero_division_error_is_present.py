# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_zero_division_error_is_present"
# subject = "builtins.ZeroDivisionError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.ZeroDivisionError: api_zero_division_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "ZeroDivisionError")
print("api_zero_division_error_is_present OK")
