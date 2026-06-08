# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_delattr_is_present"
# subject = "builtins.delattr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.delattr: api_delattr_is_present (surface)."""
import builtins

assert hasattr(builtins, "delattr")
print("api_delattr_is_present OK")
