# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_float_is_present"
# subject = "builtins.float"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.float: api_float_is_present (surface)."""
import builtins

assert hasattr(builtins, "float")
print("api_float_is_present OK")
