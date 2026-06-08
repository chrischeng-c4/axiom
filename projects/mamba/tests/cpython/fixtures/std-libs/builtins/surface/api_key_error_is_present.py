# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_key_error_is_present"
# subject = "builtins.KeyError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.KeyError: api_key_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "KeyError")
print("api_key_error_is_present OK")
