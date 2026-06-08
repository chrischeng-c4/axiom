# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_int_is_present"
# subject = "builtins.int"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.int: api_int_is_present (surface)."""
import builtins

assert hasattr(builtins, "int")
print("api_int_is_present OK")
