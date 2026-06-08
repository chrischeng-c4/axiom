# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_set_is_present"
# subject = "builtins.set"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.set: api_set_is_present (surface)."""
import builtins

assert hasattr(builtins, "set")
print("api_set_is_present OK")
