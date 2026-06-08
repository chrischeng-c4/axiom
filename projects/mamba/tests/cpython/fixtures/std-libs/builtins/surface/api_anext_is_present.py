# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_anext_is_present"
# subject = "builtins.anext"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.anext: api_anext_is_present (surface)."""
import builtins

assert hasattr(builtins, "anext")
print("api_anext_is_present OK")
