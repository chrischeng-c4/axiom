# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_value_error_is_present"
# subject = "builtins.ValueError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.ValueError: api_value_error_is_present (surface)."""
import builtins

assert hasattr(builtins, "ValueError")
print("api_value_error_is_present OK")
