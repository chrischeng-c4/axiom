# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_system_error_is_present"
# subject = "builtins.SystemError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.SystemError: api_system_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "SystemError")
print("api_system_error_is_present OK")
