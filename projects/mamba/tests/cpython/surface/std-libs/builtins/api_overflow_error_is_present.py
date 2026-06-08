# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_overflow_error_is_present"
# subject = "builtins.OverflowError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.OverflowError: api_overflow_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "OverflowError")
print("api_overflow_error_is_present OK")
