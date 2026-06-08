# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_oct_is_present"
# subject = "builtins.oct"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.oct: api_oct_is_present (surface)."""
import builtins

assert hasattr(builtins, "oct")
print("api_oct_is_present OK")
