# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_arithmetic_error_is_present"
# subject = "builtins.ArithmeticError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.ArithmeticError: api_arithmetic_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "ArithmeticError")
print("api_arithmetic_error_is_present OK")
