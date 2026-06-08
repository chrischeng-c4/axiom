# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_bytearray_is_present"
# subject = "builtins.bytearray"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.bytearray: api_bytearray_is_present (surface)."""
import builtins

assert hasattr(builtins, "bytearray")
print("api_bytearray_is_present OK")
