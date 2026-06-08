# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_runtime_error_is_present"
# subject = "builtins.RuntimeError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.RuntimeError: api_runtime_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "RuntimeError")
print("api_runtime_error_is_present OK")
