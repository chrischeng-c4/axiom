# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_max_is_present"
# subject = "builtins.max"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.max: api_max_is_present (surface)."""
import builtins

assert hasattr(builtins, "max")
print("api_max_is_present OK")
