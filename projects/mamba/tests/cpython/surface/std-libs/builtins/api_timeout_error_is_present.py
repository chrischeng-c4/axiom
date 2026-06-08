# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_timeout_error_is_present"
# subject = "builtins.TimeoutError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.TimeoutError: api_timeout_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "TimeoutError")
print("api_timeout_error_is_present OK")
