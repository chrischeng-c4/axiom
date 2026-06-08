# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_not_implemented_is_present"
# subject = "builtins.NotImplemented"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.NotImplemented: api_not_implemented_is_present (surface)."""
import builtins

assert hasattr(builtins, "NotImplemented")
print("api_not_implemented_is_present OK")
