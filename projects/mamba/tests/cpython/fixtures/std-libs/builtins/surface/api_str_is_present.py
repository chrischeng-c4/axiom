# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_str_is_present"
# subject = "builtins.str"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.str: api_str_is_present (surface)."""
import builtins

assert hasattr(builtins, "str")
print("api_str_is_present OK")
