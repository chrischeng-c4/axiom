# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_object_is_present"
# subject = "builtins.object"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.object: api_object_is_present (surface)."""
import builtins

assert hasattr(builtins, "object")
print("api_object_is_present OK")
