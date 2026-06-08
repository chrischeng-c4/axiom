# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_interrupted_error_is_present"
# subject = "builtins.InterruptedError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.InterruptedError: api_interrupted_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "InterruptedError")
print("api_interrupted_error_is_present OK")
