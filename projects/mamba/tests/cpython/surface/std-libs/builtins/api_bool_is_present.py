# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_bool_is_present"
# subject = "builtins.bool"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.bool: api_bool_is_present (surface)."""
import builtins

assert hasattr(builtins, "bool")
print("api_bool_is_present OK")
