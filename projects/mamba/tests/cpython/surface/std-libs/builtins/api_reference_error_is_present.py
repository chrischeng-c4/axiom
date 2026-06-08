# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_reference_error_is_present"
# subject = "builtins.ReferenceError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.ReferenceError: api_reference_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "ReferenceError")
print("api_reference_error_is_present OK")
