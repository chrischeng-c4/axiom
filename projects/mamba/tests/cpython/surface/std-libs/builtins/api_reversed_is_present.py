# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_reversed_is_present"
# subject = "builtins.reversed"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.reversed: api_reversed_is_present (surface)."""
import builtins

assert hasattr(builtins, "reversed")
print("api_reversed_is_present OK")
