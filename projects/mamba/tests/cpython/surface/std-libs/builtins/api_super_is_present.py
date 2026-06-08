# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_super_is_present"
# subject = "builtins.super"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.super: api_super_is_present (surface)."""
import builtins

assert hasattr(builtins, "super")
print("api_super_is_present OK")
