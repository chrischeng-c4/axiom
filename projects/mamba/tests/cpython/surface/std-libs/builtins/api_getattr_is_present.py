# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_getattr_is_present"
# subject = "builtins.getattr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.getattr: api_getattr_is_present (surface)."""
import builtins

assert hasattr(builtins, "getattr")
print("api_getattr_is_present OK")
