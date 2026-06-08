# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_environment_error_is_present"
# subject = "builtins.EnvironmentError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.EnvironmentError: api_environment_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "EnvironmentError")
print("api_environment_error_is_present OK")
